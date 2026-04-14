import { useEffect, useState } from "react"
import { Link } from "wouter"
import { useProviderStore } from "../../stores/providerStore"
import { prefetchAlbum } from "../../stores/metadataCache"
import { useAlbumImage } from "../../hooks/useMediaImage"
import { MediaGrid } from "../shared/MediaGrid"
import { useContextMenu } from "../../hooks/useContextMenu"
import type { MusicAlbum } from "../../types/music"


function AlbumThumb({ artist, title, thumb }: { artist: string; title: string; thumb: string | null }) {
  const resolved = useAlbumImage(artist, title, thumb)
  if (resolved) {
    return (
      <img
        src={resolved}
        alt={title}
        loading="lazy"
        className="h-full w-full object-cover transition-transform group-hover:scale-105"
      />
    )
  }
  return (
    <div className="flex h-full w-full items-center justify-center">
      <svg viewBox="0 0 24 24" width="40" height="40" fill="#535353">
        <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
      </svg>
    </div>
  )
}

export function AllAlbumsPage() {
  const provider = useProviderStore(s => s.provider)
  const { handler: ctxMenu } = useContextMenu()
  const [albums, setAlbums] = useState<MusicAlbum[]>([])
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    if (!provider?.getAllAlbums) return
    setIsLoading(true)
    provider.getAllAlbums(0, 5000).then(result => {
      setAlbums(result.items)
      setIsLoading(false)
    })
  }, [provider])

  return (
    <div className="pb-12">
      <div className="flex flex-row items-end p-8">
        <div className="flex w-60 h-60 flex-shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-cyan-700 to-blue-500 shadow-2xl">
          <svg viewBox="0 0 24 24" width="80" height="80" fill="white">
            <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" />
          </svg>
        </div>
        <div className="pl-6 flex flex-col justify-between flex-1 h-60 min-w-0">
          <div>
            <div className="whitespace-nowrap text-[76px] font-black leading-none">
              All Albums
            </div>
            <p className="mt-2 max-w-xl select-text text-sm text-gray-400">
              Every album in your library.
            </p>
          </div>
          <p className="text-sm text-gray-400">
            {albums.length} {albums.length === 1 ? "album" : "albums"}
          </p>
        </div>
      </div>

      <div className="px-8 pt-2">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-white/20 border-t-accent" />
          </div>
        ) : albums.length === 0 ? (
          <div className="py-12 text-center text-sm text-gray-500">
            No albums found in your library.
          </div>
        ) : (
          <MediaGrid>
            {albums.map(album => (
              <Link
                key={album.id}
                href={`/album/${album.id}`}
                onMouseEnter={() => prefetchAlbum(album.id)}
                onContextMenu={ctxMenu("album", album)}
                className="group flex flex-col gap-2 rounded-md p-3 no-underline transition-colors hover:bg-hl-card"
              >
                <div className="relative w-full aspect-square overflow-hidden rounded-md bg-app-surface shadow-lg">
                  <AlbumThumb artist={album.artistName} title={album.title} thumb={album.thumbUrl} />
                </div>
                <div className="w-full min-w-0">
                  <div className="truncate font-semibold text-sm text-white">
                    {album.title}
                  </div>
                  <div className="truncate text-xs text-gray-400">
                    {album.artistName}
                    {album.year ? ` · ${album.year}` : ""}
                  </div>
                </div>
              </Link>
            ))}
          </MediaGrid>
        )}
      </div>
    </div>
  )
}
