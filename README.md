# sacklite
yet another LBP custom server implementation in rust made for the hell of it

NOTE: this is extremely WIP and security isn't entirely there yet, please don't run a public instance with this thanks

# implemented so far
- NpTicket authentication, including signature + expiry verification
- resource uploading/downloading (but still no filetype checks)
- user stuff (bio, pins, icon, comments)
- level stuff (publishing, updating, comments, hearts, queue)
- autodiscover API from Refresh/Bunkum