# The Serva OS

## Services

The model of a client and server can be found almost everywhere.
Serva takes this pattern to the extreme: "What if all programs are services?"
The key motivation behind this radical approach is because it allows the kernel
to validate that user-code runs as specified in a [pre-defined contract](service-spec.md) (_service spec_).
As apposed to Unix, programmers can assume that the system is installed and running as expected before
their code even starts executing.

## Privileges and _(the lack of)_ users

Each service runs with a so-called _privilege level_, these levels are (in descending order):

1. Kernel
2. System
3. User

These privilege levels encapsulate all but the most obscure use-cases of Unix's users and groups.
It must be noted that services can implement their own definition of what it means to be a "user of the system."

## Files

Because a service is such a powerful interface, Serva completely gets rid of
the [VFS](https://en.wikipedia.org/wiki/Virtual_file_system)!
You may begin to question in great dismay: "But how do I read and write to disk? And why'd you get rid of my pipes!"

1. The kernel exposes a _data_ service, which returns a stream descriptor.
2. You can share stream descriptors.