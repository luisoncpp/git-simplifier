# Base chooser must refresh when editing

The initial repository load can skip Base-choice discovery once a Base is configured. A later **Change Base** action must therefore fetch `refs/remotes/*` explicitly before opening the selector; an empty cached list means “not loaded,” not “no remote-tracking refs exist.”
