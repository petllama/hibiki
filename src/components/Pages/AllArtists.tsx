import { useEffect, useState } from "react"
import { Link } from "wouter"
import { useProviderStore } from "../../stores/providerStore"
import { prefetchArtist } from "../../stores/metadataCache"
import { useArtistImage } from "../../hooks/useMediaImage"
import { MediaGrid } from "../shared/MediaGrid"
import { useContextMenu } from "../../hooks/useContextMenu"
import type { MusicArtist } from "../../types/music"


function ArtistThumb({ title, thumb }: { title: string; thumb: string | null }) {
  const resolved = useArtistImage(title, thumb)
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
        <path d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z" />
      </svg>
    </div>
  )
}

export function AllArtistsPage() {
  const provider = useProviderStore(s => s.provider)
  const { handler: ctxMenu } = useContextMenu()
  const [artists, setArtists] = useState<MusicArtist[]>([])
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    if (!provider?.getAllArtists) return
    setIsLoading(true)
    provider.getAllArtists().then(result => {
      setArtists(result.items)
      setIsLoading(false)
    })
  }, [provider])

  return (
    <div className="pb-12">
      <div className="flex flex-row items-end p-8">
        <div className="flex w-32 h-32 flex-shrink-0 items-center justify-center rounded-md bg-gradient-to-br from-indigo-700 to-violet-500 shadow-2xl">
          <svg viewBox="0 0 24 24" width="48" height="48" fill="white">
            <path d="M12 12c2.7 0 4.8-2.1 4.8-4.8S14.7 2.4 12 2.4 7.2 4.5 7.2 7.2 9.3 12 12 12zm0 2.4c-3.2 0-9.6 1.6-9.6 4.8v2.4h19.2v-2.4c0-3.2-6.4-4.8-9.6-4.8z" />
          </svg>
        </div>
        <div className="pl-6 flex flex-col justify-end flex-1 h-32 min-w-0">
          <div className="text-3xl font-black leading-tight">All Artists</div>
          <p className="mt-1 text-sm text-gray-400">
            {artists.length} {artists.length === 1 ? "artist" : "artists"}
          </p>
        </div>
      </div>

      <div className="px-8 pt-2">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="h-8 w-8 animate-spin rounded-full border-2 border-white/20 border-t-accent" />
          </div>
        ) : artists.length === 0 ? (
          <div className="py-12 text-center text-sm text-gray-500">
            No artists found in your library.
          </div>
        ) : (
          <MediaGrid>
            {artists.map(artist => (
              <Link
                key={artist.id}
                href={`/artist/${artist.id}`}
                onMouseEnter={() => prefetchArtist(artist.id)}
                onContextMenu={ctxMenu("artist", artist)}
                className="group flex flex-col items-center gap-2 rounded-md p-3 no-underline transition-colors hover:bg-hl-card"
              >
                <div className="relative w-full aspect-square overflow-hidden rounded-full bg-app-surface shadow-lg">
                  <ArtistThumb title={artist.title} thumb={artist.thumbUrl} />
                </div>
                <div className="w-full text-center">
                  <div className="truncate font-semibold text-sm text-white group-hover:text-white">
                    {artist.title}
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
