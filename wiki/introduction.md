# The Serva OS

## Services

The model of a client and server can be found almost everywhere.
Serva takes this pattern to the extreme: "What if all programs are services?"
The key motivation behind this radical approach is because it allows the kernel
to validate that user-code runs as specified in a [pre-defined contract](service-spec.md) (service spec).
Another stability benefit of the service model is that the kernel automatically can manage the [lifecycle](lifecycle.md)
of each service.
As apposed to Unix, programmers can assume that the system is installed and running as expected before
their code even starts executing.

## Privileges and _(the lack of)_ users

Each service runs with a so-called _privilege level_, these levels are (in descending order):

| Name   | Limitations and Powers                                                                                                     | Use-case                                                                  |
|:-------|:---------------------------------------------------------------------------------------------------------------------------|:--------------------------------------------------------------------------|
| Kernel | Kernel privileges and is mapped with the kernel, because the latter, memory is constrained.                                | Drivers                                                                   |
| System | Can also make requests not specified in the service spec. These powers only apply to services of equal or lower privilege. | System management, equivalent of the root user                            |
| User   | Exactly like _System_ but one privilege level lower.                                                                       | A user that can use the system without the fear of destroying said system |
| Local  | Can only make request to services specified in the service spec.                                                           | Everything else                                                           |

These privilege levels encapsulate all but the most obscure use-cases of Unix's users and groups.
It must be noted that services can implement their own definition of what it means to be a "user of the system".

## Files

Because a service is such a powerful interface, Serva completely gets rid of
the [VFS](https://en.wikipedia.org/wiki/Virtual_file_system)!
You begin to question in great dismay: "But how do I read and write to disk? And why'd you get rid of my pipes!"

1. The kernel exposes a _data_ service, which returns a stream descriptor.
2. You can share stream descriptors.

## Drivers

The Serva kernel is an exo-kernel.
This means that services can run directly in Ring0.
If you wish to write a kernel service, simply change the privilege level in the service spec.