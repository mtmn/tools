use crate::fetchers::discogs::DiscogsClient;
use crate::fetchers::musicbrainz::MusicBrainzClient;

pub struct AppContext {
    pub mb_client: Option<MusicBrainzClient>,
    pub discogs_client: Option<DiscogsClient>,
}
