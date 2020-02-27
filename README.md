# bideorai
A web frontend and server for viewing a personal video library. 

## Why?
I get unexpectedly bad video playback for some video streaming services, despite having a high downspeed.
I mostly blame Comcast. I believe Chrome is complicit, since it absolutely refuses to buffer a
reasonable amount of content. I've seen videos buffer while being served _literally_ from the
same (otherwise unencumbered) machine. This is a particular issue with adaptive streaming. This results in
a high rebuffering time, switching to a lower bitrate stream, and audio popping. As far as I
can tell, there's no way 

Yeah, so I'm doing all of this because websites/browsers don't buffer properly.

## Planned Features
- Buffering significant portions of the video
    - If I'm watching a video, I'm probably watching a significant chunk of it.
      There are few savings to be had from avoiding buffering.
- Hosted on the public internet so that I'm not dependent on a particular
  device or network
- File delivery from backblaze b2
    - I need to store the content somewhere and b2 is the cheapest option

## Non-Features
These are things which I explicitly _do not_ intend to implement, since they
do not benefit _my particular_ application.
- Adaptive streaming / multiple bitrate sources
    - This makes delivery and storage more complicated and expensive
    - I will only watch videos on a reasonably steady connection

# Screw it, the file system is our database!
No DBMS? What about S C A L I N G?

We don't need 5 nines. Data can be lost without issues. Everything is running
from a single node. SSD's are fast. Using a database is unnecessary.
