#!/usr/bin/env node

import fs from 'node:fs'
import { execFileSync } from 'node:child_process'

const VERSION_PATTERN = /^v?(\d+)\.(\d+)\.(\d+)(?:[-+][0-9A-Za-z.-]+)?$/

function commandName(command) {
  if (process.platform === 'win32' && ['vp', 'vpr'].includes(command)) {
    return `${command}.cmd`
  }

  return command
}

function run(command, args = [], options = {}) {
  const printable = [command, ...args].join(' ')

  try {
    return execFileSync(commandName(command), args, {
      encoding: 'utf8',
      stdio: options.silent ? 'pipe' : 'inherit',
      cwd: options.cwd,
    })
  } catch (error) {
    throw new Error(`Command failed: ${printable}\n${error.message}`)
  }
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'))
}

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`, 'utf8')
}

function normalizeVersion(version) {
  const match = version.match(VERSION_PATTERN)
  if (!match) return null

  return {
    cleanVersion: version.replace(/^v/, ''),
    tagVersion: version.startsWith('v') ? version : `v${version}`,
  }
}

function parseArgs(argv) {
  const flags = new Set(argv.filter(arg => arg.startsWith('--')))
  const positional = argv.filter(arg => !arg.startsWith('--'))

  return {
    version: positional[0],
    help: flags.has('--help') || flags.has('-h'),
    skipChecks: flags.has('--skip-checks'),
    skipInstall: flags.has('--skip-install'),
    allowDirty: flags.has('--allow-dirty'),
  }
}

function showHelp() {
  console.log(`
Prepare a release by updating project version files and validating release config.

Usage:
  vpr release:prepare v1.0.0
  vpr release:prepare 1.0.0

Options:
  --skip-checks   Do not run vpr check:all or cargo check
  --skip-install  Do not update package-lock.json
  --allow-dirty   Allow running with uncommitted changes
  --help          Show this help message

Updated files:
  package.json
  package-lock.json
  src-tauri/Cargo.toml
  src-tauri/tauri.conf.json
`)
}

function assertCleanGitStatus({ allowDirty }) {
  if (allowDirty) {
    console.warn(
      'WARN: Skipping clean worktree check because --allow-dirty was set.'
    )
    return
  }

  const gitStatus = run('git', ['status', '--porcelain'], { silent: true })
  if (gitStatus.trim()) {
    throw new Error(
      [
        'Working directory is not clean. Commit or stash changes first,',
        'or rerun with --allow-dirty if this is intentional.',
        '',
        gitStatus.trim(),
      ].join('\n')
    )
  }
}

function updatePackageJson(cleanVersion) {
  const filePath = 'package.json'
  const pkg = readJson(filePath)
  const oldVersion = pkg.version
  pkg.version = cleanVersion
  writeJson(filePath, pkg)
  console.log(`package.json: ${oldVersion} -> ${cleanVersion}`)
}

function updateCargoToml(cleanVersion) {
  const filePath = 'src-tauri/Cargo.toml'
  const cargoToml = fs.readFileSync(filePath, 'utf8')
  const oldVersion = cargoToml.match(/^version = "([^"]+)"/m)?.[1] ?? 'unknown'

  if (!/^version = "[^"]+"/m.test(cargoToml)) {
    throw new Error('Could not find package version in src-tauri/Cargo.toml')
  }

  const updatedCargoToml = cargoToml.replace(
    /^version = "[^"]+"/m,
    `version = "${cleanVersion}"`
  )

  fs.writeFileSync(filePath, updatedCargoToml, 'utf8')
  console.log(`src-tauri/Cargo.toml: ${oldVersion} -> ${cleanVersion}`)
}

function updateTauriConfig(cleanVersion) {
  const filePath = 'src-tauri/tauri.conf.json'
  const tauriConfig = readJson(filePath)
  const oldVersion = tauriConfig.version
  tauriConfig.version = cleanVersion
  writeJson(filePath, tauriConfig)
  console.log(`src-tauri/tauri.conf.json: ${oldVersion} -> ${cleanVersion}`)
  return tauriConfig
}

function updateLockfile({ skipInstall }) {
  if (skipInstall) {
    console.warn(
      'WARN: Skipping package-lock.json update because --skip-install was set.'
    )
    return
  }

  run('vp', ['install', '--lockfile-only', '--ignore-scripts'])
  console.log('package-lock.json updated')
}

function verifyReleaseConfig(tauriConfig) {
  const warnings = []

  if (!tauriConfig.bundle?.createUpdaterArtifacts) {
    warnings.push('createUpdaterArtifacts is not enabled in tauri.conf.json')
  }

  const pubkey = tauriConfig.plugins?.updater?.pubkey
  if (!pubkey || pubkey.includes('YOUR_')) {
    warnings.push('Updater public key is not configured')
  }

  const endpoints = tauriConfig.plugins?.updater?.endpoints ?? []
  if (
    endpoints.length === 0 ||
    endpoints.some(endpoint => endpoint.includes('YOUR_USERNAME/YOUR_REPO'))
  ) {
    warnings.push('Updater endpoint still contains template placeholders')
  }

  const publisher = tauriConfig.bundle?.publisher
  if (!publisher || publisher === 'Your Name') {
    warnings.push('Bundle publisher still contains the template value')
  }

  if (warnings.length === 0) {
    console.log('Release configuration looks ready')
    return
  }

  console.warn('Release configuration warnings:')
  for (const warning of warnings) {
    console.warn(`- ${warning}`)
  }
}

function runChecks({ skipChecks }) {
  if (skipChecks) {
    console.warn('WARN: Skipping release checks because --skip-checks was set.')
    return
  }

  run('vpr', ['check:all'])
  run('cargo', ['check'], { cwd: 'src-tauri' })
}

function getCurrentBranch() {
  try {
    return run('git', ['branch', '--show-current'], { silent: true }).trim()
  } catch {
    return 'main'
  }
}

function printNextSteps(tagVersion) {
  const branch = getCurrentBranch() || 'main'

  console.log(`\nRelease ${tagVersion} is prepared.`)
  console.log('\nReview the diff, then run:')
  console.log(
    '  git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json'
  )
  console.log(`  git commit -m "chore: release ${tagVersion}"`)
  console.log(`  git tag ${tagVersion}`)
  console.log(`  git push origin ${branch}`)
  console.log(`  git push origin ${tagVersion}`)
}

function prepareRelease() {
  const args = parseArgs(process.argv.slice(2))

  if (args.help) {
    showHelp()
    return
  }

  const version = args.version ? normalizeVersion(args.version) : null
  if (!version) {
    showHelp()
    throw new Error('Missing or invalid version. Expected a value like v1.0.0.')
  }

  console.log(`Preparing release ${version.tagVersion}`)

  assertCleanGitStatus(args)
  runChecks(args)
  updatePackageJson(version.cleanVersion)
  updateCargoToml(version.cleanVersion)
  const tauriConfig = updateTauriConfig(version.cleanVersion)
  updateLockfile(args)
  verifyReleaseConfig(tauriConfig)
  runChecks(args)
  printNextSteps(version.tagVersion)
}

try {
  prepareRelease()
} catch (error) {
  console.error(`\nRelease preparation failed: ${error.message}`)
  process.exit(1)
}
