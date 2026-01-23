#include <mpd/client.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
// #define DEBUG
#ifdef DEBUG
#define D(x)                                                                   \
  do {                                                                         \
    x;                                                                         \
  } while (0)
#else
#define D(x)                                                                   \
  do {                                                                         \
  } while (0)
#endif
#define DEFAULT_HOST "localhost"
#define DEFAULT_PORT 6600
struct mpd_connection *conn() {
  // Read host from environment variable, use default if not set
  const char *host = getenv("MPD_HOST");
  if (host == NULL || strlen(host) == 0) {
    host = DEFAULT_HOST;
  }
  D(printf("Using host: %s\n", host));

  // Read port from environment variable, use default if not set
  unsigned port = DEFAULT_PORT;
  const char *port_str = getenv("MPD_PORT");
  if (port_str != NULL && strlen(port_str) > 0) {
    port = (unsigned)atoi(port_str);
  }
  D(printf("Using port: %u\n", port));

  D(printf("%s %s:%u\n", "Connecting to", host, port));

  struct mpd_connection *c = mpd_connection_new(host, port, 0);
  enum mpd_error err = mpd_connection_get_error(c);
  if (err != 0) {
    printf("Error code: %u. View error codes here: "
           "https://www.musicpd.org/doc/libmpdclient/error_8h.html\n",
           err);
    mpd_connection_free(c);
    return 0;
  }
#ifdef PASS
  const char *pass = PASS;
  if (mpd_run_password(c, pass) == false) {
    printf("%s\n", "Bad password");
    mpd_connection_free(c);
    return 0;
  }
#endif
  D(printf("%s %s:%u\n", "Connected to", host, port));
  return c;
}
int main(int argc, char *argv[]) {
  // Check for playlist name argument
  if (argc < 2) {
    printf("Usage: %s PLAYLIST_NAME\n", argv[0]);
    return 1;
  }

  const char *playlist = argv[1];
  D(printf("Using playlist: %s\n", playlist));

  struct mpd_connection *c = conn();
  if (c == 0)
    return -1;

  struct mpd_song *curr = mpd_run_current_song(c);
  if (curr == NULL) {
    printf("No song is currently playing\n");
    mpd_connection_free(c);
    return -1;
  }

  const char *curr_uri = mpd_song_get_uri(curr);
  D(printf("Currently playing: %s\n", curr_uri));

  if (mpd_run_playlist_add(c, playlist, curr_uri)) {
    printf("%s %s %s %s\n", "Added", curr_uri, "to playlist", playlist);
  } else {
    printf("%s\n", "Some error");
    mpd_song_free(curr);
    mpd_connection_free(c);
    return -1;
  }
  // Free resources
  mpd_song_free(curr);
  mpd_connection_free(c);

  return 0;
}
