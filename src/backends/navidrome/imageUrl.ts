/**
 * Navidrome/Subsonic image URL assembly.
 *
 * Builds the Subsonic getCoverArt URL then wraps it in the semantic
 * `image://` scheme via the shared buildImageUrl().
 */

import { buildImageUrl } from "../../lib/imageUrl"

type EntityType = "artist" | "album" | "track" | "playlist"

/**
 * Build an image URL from a Subsonic cover art URL.
 *
 * The cover art URL is already a fully authenticated URL from the Rust backend,
 * so we just wrap it in the image:// scheme.
 */
export function buildNavidromeImageUrl(
  coverArtUrl: string,
  entityType: EntityType,
  entityId: string,
  name?: string | null,
  artist?: string | null,
): string | null {
  if (!coverArtUrl) return null
  return buildImageUrl(entityType, entityId, coverArtUrl, name, artist)
}
