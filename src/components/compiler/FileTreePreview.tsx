import { FileArchive, FileCode2, Folder, FolderTree } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useCompilerStore } from '@/store/compiler-store'
import type { BuildConfiguration, BuildRequest } from '@/lib/tauri-bindings'
import { HelpHint } from './HelpHint'

interface TreeNode {
  name: string
  kind: 'folder' | 'file'
  fileType?: 'zip' | 'binary' | 'note'
  children?: TreeNode[]
}

export function FileTreePreview() {
  const request = useCompilerStore(state => state.request)
  const tree = buildPreviewTree(request)

  return (
    <>
      <div className="border-b px-4 py-3">
        <div className="flex items-center gap-2 text-sm font-semibold">
          <FolderTree className="size-4" />
          Output Preview
          <HelpHint text="Preview of package folders, binaries, copied resources, and zip files generated from the current build settings." />
        </div>
        <div className="mt-1 flex flex-wrap gap-1.5">
          {request.versions.map(version => (
            <Badge key={version} variant="outline">
              {version}
            </Badge>
          ))}
        </div>
      </div>
      <ScrollArea className="flex-1">
        <div className="space-y-3 p-4">
          <div className="rounded-md border bg-muted/20 p-3">
            <div className="text-xs text-muted-foreground">Output Dir</div>
            <div className="mt-1 break-all font-mono text-xs">
              {request.output_dir || `${request.plugin_root}\\dist`}
            </div>
          </div>
          <div className="space-y-1 font-mono text-xs">
            <TreeNodeView node={tree} depth={0} />
          </div>
        </div>
      </ScrollArea>
    </>
  )
}

function TreeNodeView({ node, depth }: { node: TreeNode; depth: number }) {
  const Icon =
    node.kind === 'folder'
      ? Folder
      : node.fileType === 'zip'
        ? FileArchive
        : FileCode2
  return (
    <div>
      <div
        className="flex min-w-0 items-center gap-2 rounded px-1.5 py-1"
        style={{ paddingLeft: `${depth * 14 + 6}px` }}
      >
        <Icon className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="truncate">{node.name}</span>
      </div>
      {node.children?.map(child => (
        <TreeNodeView
          key={`${depth}-${child.name}`}
          node={child}
          depth={depth + 1}
        />
      ))}
    </div>
  )
}

function buildPreviewTree(request: BuildRequest): TreeNode {
  const outputName = lastPathPart(request.output_dir || 'dist')
  const children: TreeNode[] = []

  if (request.package_mode === 'Merged' || request.package_mode === 'Both') {
    children.push({
      name: request.package_name || 'Package',
      kind: 'folder',
      children: [
        {
          name: 'res',
          kind: 'folder',
          children: [
            {
              name: 'copied from plugin root if present',
              kind: 'file',
              fileType: 'note',
            },
          ],
        },
        ...request.versions.flatMap(version =>
          buildConfigurations(request.configuration).map(configuration => ({
            name: `${request.package_name || 'Plugin'} ${version} ${configuration}.xdl64`,
            kind: 'file' as const,
            fileType: 'binary' as const,
          }))
        ),
      ],
    })

    if (request.zip_enabled) {
      children.push({
        name: `${request.package_name || 'Package'}.zip`,
        kind: 'file',
        fileType: 'zip',
      })
    }
  }

  if (
    request.package_mode === 'PerVersion' ||
    request.package_mode === 'Both'
  ) {
    for (const version of request.versions) {
      for (const configuration of buildConfigurations(request.configuration)) {
        const folderName = `${request.package_name || 'Package'}_${version}_${configuration}`
        children.push({
          name: folderName,
          kind: 'folder',
          children: [
            {
              name: 'res',
              kind: 'folder',
              children: [
                {
                  name: 'copied from plugin root if present',
                  kind: 'file',
                  fileType: 'note',
                },
              ],
            },
            {
              name: `${request.package_name || 'Plugin'} ${version}.xdl64`,
              kind: 'file',
              fileType: 'binary',
            },
          ],
        })
        if (request.zip_enabled) {
          children.push({
            name: `${folderName}.zip`,
            kind: 'file',
            fileType: 'zip',
          })
        }
      }
    }
  }

  return {
    name: outputName,
    kind: 'folder',
    children,
  }
}

function buildConfigurations(configuration: BuildConfiguration) {
  if (configuration === 'Both') {
    return ['Debug', 'Release']
  }
  return [configuration]
}

function lastPathPart(path: string) {
  return path.split(/[\\/]/).filter(Boolean).at(-1) ?? path
}
