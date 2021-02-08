# RSSmission
RSSmission let's you automatically add torrents from RSS feeds to your transmission daemon.
It was inspired by the great [transmission-rss](https://github.com/nning/transmission-rss) project that i unfortunately couldn't get to work reliably.

## Building
Before building make sure you have a working rust and cargo installation.
```
git clone https://github.com/veselysps/rssmission
cd rssmission
cargo build --release
```

## Config
Place your config in the directory that you are planning to run RSSmission from and make sure its name `rssmission.json`. 
### Example
```json
{
    "server": {
        "url": "http://127.0.0.1:51413/transmission/rpc", // make sure to include the full rpc path
        "username": "username", // your transmission username
        "password": "password" // your transmission password
    },
    "feeds": [ 
        {
            "url": "https://example.com/feed0.xml", // link to your rss feed
            "matchers": [
                {
                    "regexp": "*", // to match all torrents and downloaded them to the default transmission
                                   // directory you can user regex * without any specified path
                },
            ]
        },
        {
            "url": "https://example.com/feed1.xml",
            "matchers": [
                {
                    "regexp": "Match1", // you can use any valid regex here 
                    "path": "/home/marek/torrents/Match1" // download path of the torrent
                },
                {
                    "regexp": "\\(Match2\\)", // if you need to escape a character
                                              // ( and ) in this case you need to use \\ instead of \
                    "path": "/home/marek/torrents/Match2"
                }
            ]
        }
    ]
}
```

## Running automatically
Currently the only way to run this automatically is using a cron job, I believe you can set it up yourself. Here's a little tool to help you figure it out [crontab.guru](https://crontab.guru).