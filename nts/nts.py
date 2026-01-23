#!/usr/bin/env python3
import argparse
import requests
import sys
import json
from typing import Optional, Dict
from urllib.parse import urlparse, quote

class NTSTracklistFetcher:
    def __init__(self):
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.114 Safari/537.36',
            'Accept': 'application/json'
        })
    
    def extract_episode_path(self, url: str) -> Optional[str]:
        """Extract show and episode path from URL."""
        try:
            parsed = urlparse(url)
            path_parts = [p for p in parsed.path.split('/') if p]
            
            if len(path_parts) >= 4 and path_parts[0] == 'shows' and path_parts[2] == 'episodes':
                show_name = path_parts[1]
                episode_name = path_parts[3]
                return f"{show_name}/episodes/{episode_name}"
            return None
        except Exception as e:
            print(f"Error extracting path: {e}", file=sys.stderr)
            return None
    
    def extract_episode_name(self, url: str) -> Optional[str]:
        """Extract episode name for filename from URL."""
        try:
            parsed = urlparse(url)
            path_parts = [p for p in parsed.path.split('/') if p]
            
            if len(path_parts) >= 4 and path_parts[0] == 'shows' and path_parts[2] == 'episodes':
                return path_parts[3]
            return None
        except Exception as e:
            print(f"Error extracting episode name: {e}", file=sys.stderr)
            return None
    
    def get_episode_data(self, episode_path: str) -> Dict:
        """Fetch episode data from the API."""
        encoded_path = '/'.join(quote(part) for part in episode_path.split('/'))
        url = f"https://www.nts.live/api/v2/shows/{encoded_path}"
        
        if '--debug' in sys.argv:
            print(f"DEBUG: Requesting URL: {url}", file=sys.stderr)
        
        response = self.session.get(url)
        response.raise_for_status()
        return response.json()
    
    def save_tracklist(self, episode_data: Dict, filename: str) -> None:
        """Save tracks to a text file."""
        if '--debug' in sys.argv:
            print(f"DEBUG: Response data: {json.dumps(episode_data, indent=2)}", file=sys.stderr)
        
        # Extract tracks from the embeds.tracklist.results path
        tracklist = episode_data.get('embeds', {}).get('tracklist', {}).get('results', [])
        
        if not tracklist:
            print("No tracks found in this episode", file=sys.stderr)
            return
        
        with open(filename, 'w', encoding='utf-8') as f:
            for track in tracklist:
                artist = track.get('artist', 'Unknown Artist')
                title = track.get('title', 'Unknown Title')
                f.write(f"{artist} - {title}\n")
        
        print(f"Tracklist saved to {filename}")
    
    def fetch_and_save_tracklist(self, url: str) -> None:
        """Main method to fetch and save tracklist."""
        episode_path = self.extract_episode_path(url)
        episode_name = self.extract_episode_name(url)
        
        if not episode_path or not episode_name:
            raise ValueError("Could not extract episode information from URL")
        
        filename = f"{episode_name}.md"
        
        if '--debug' in sys.argv:
            print(f"DEBUG: Extracted path: {episode_path}", file=sys.stderr)
            print(f"DEBUG: Output filename: {filename}", file=sys.stderr)
        
        try:
            episode_data = self.get_episode_data(episode_path)
            self.save_tracklist(episode_data, filename)
        except requests.exceptions.HTTPError as e:
            error_msg = f"Error fetching episode data: {str(e)}"
            raise
        except Exception as e:
            print(f"Error processing episode: {str(e)}", file=sys.stderr)
            raise

def main():
    parser = argparse.ArgumentParser(description='Fetch tracklist from NTS Radio shows')
    parser.add_argument('url', help='URL of the NTS Radio show')
    parser.add_argument('--debug', action='store_true', help='Enable debug output')
    
    args = parser.parse_args()
    
    try:
        fetcher = NTSTracklistFetcher()
        fetcher.fetch_and_save_tracklist(args.url)
    except requests.exceptions.RequestException:
        sys.exit(1)
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception:
        sys.exit(1)

if __name__ == '__main__':
    main()
