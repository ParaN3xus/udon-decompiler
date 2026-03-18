#import "/docs/book.typ": book-page, cross-link

#show: book-page.with(title: "Reporting Issues")

*This document was translated from Chinese by an LLM.*


If you encounter crashes, incorrect generated code logic, or other unexpected behavior while using the project, you can report issues as follows.

= Locate the failing file
In addition to decompiling an entire folder at once, this project also allows decompiling a single file: simply replace the folder path with the path to a `.hex` file or an `.asset` file.

When decompiling a folder fails, you can read the program logs to locate the specific failing file, for example by searching the logs for related file names.

After finding the specific failing file, you can re-run decompilation on that file to confirm the issue.

= Report the issue
Please create a new issue on this project's repository #link("https://github.com/ParaN3xus/udon-decompiler/issues")[issue] page, describe the problem as clearly as possible, and *provide the failing file to us*.
