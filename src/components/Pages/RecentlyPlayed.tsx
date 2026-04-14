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

export function RecentlyPlayedPage() {
  const provider = useProviderStore(s => s.provider)
  const { handler: ctxMenu } = useContextMenu()
  const [albums, setAlbums] = useState<MusicAlbum[]>([])
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    if (!provider?.getAllAlbums) return
    setIsLoading(true)
    // Subsonic "recent" type returns recently played albums
    provider.getRecentlyPlayed?.().then(result => {
      setAlbums(result)
      setIsLoading(false)
    }).catch(() => setIsLoading(false))
  }, [provider])

  return (
    <div className="pb-12">
      <div className="flex flex-row items-end p-8">
        <div className="flex w-32 h-32 flex-shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-amber-700 to-orange-500 shadow-2xl">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="white">
            <path d="M13 3a9 9 0 0 0-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42A8.954 8.954 0 0 0 13 21a9 9 0 0 0 0-18zm-1 5v5l4.28 2.54.72-1.21-3.5-2.08V8H12z" />
          </svg>
        </div>
        <div className="pl-6 flex flex-col justify-end flex-1 h-32 min-w-0">
          <div className="text-3xl font-black leading-tight">Recently Played</div>
          <p className="mt-1 text-sm text-gray-400">
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
            No recently played albums.
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
