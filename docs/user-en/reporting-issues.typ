#import "/docs/book.typ": book-page

#show: book-page.with(title: "Reporting Issues")

This document was translated by an LLM.

If you encounter crashes, incorrect generated code logic, or other unexpected behavior while using the project, you can report issues as follows.

= Locate the failing file
In addition to decompiling an entire folder at once, this project also allows decompiling a single file: just replace the folder path with the path to a `.json` file.

When a folder decompilation fails, read the program logs to locate the specific failing file, for example by searching the log for relevant file names.

After finding the specific failing file, you can re-run decompilation on that file to confirm the issue.

= Report the issue
Please create a new issue on the project #link("https://github.com/ParaN3xus/udon-decompiler/issues")[issue] page, describe the problem as clearly as possible, and provide the failing file to us.
