#import "/docs/book.typ": book-page, cross-link

#show: book-page.with(title: "汇报错误")

For English version, refer to #cross-link("/user-en/reporting-issues.typ")[Reporting Issues].

如果在使用过程中出现程序崩溃, 生成代码逻辑错误, 或者观察到其他非预期的程序行为, 你可以按如下步骤向我们汇报错误.

= 定位出错文件
除了一次性反编译一整个文件夹之外, 本项目还允许反编译单个文件: 只需要简单地把文件夹路径换成 `.json` 文件的路径即可.

当反编译某个文件夹出错时, 可以阅读程序日志以定位具体的出错文件. 定位的方法可以是在日志中搜索相关文件名等.

找到具体的出错文件后, 你可以在该文件上重新执行反编译以确认错误.

= 汇报错误
请在本项目仓库的 #link("https://github.com/ParaN3xus/udon-decompiler/issues")[issue] 页面新建一个 issue, 用尽量清晰的语言描述你所遇到的错误, 并将出错文件提供给我们.
