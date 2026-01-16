## `apply_patch`

使用 `apply_patch` 终端命令来编辑文件。
补丁语言是一种精简的、面向文件的 diff 格式，易于解析且应用安全。可以将其理解为一个高层级的封套：

*** Begin Patch
[ 一个或多个文件区块 ]
*** End Patch

封套内部由一组文件操作组成。
你必须通过头部指明要执行的动作。
每个操作都以以下三个头之一开头：

*** Add File: <path> —— 创建新文件。后续每行都以 `+` 开头，表示文件初始内容。
*** Delete File: <path> —— 删除已有文件。之后不再跟任何内容。
*** Update File: <path> —— 就地修改现有文件（可选地配合重命名）。

若需重命名，可紧跟一行 *** Move to: <new path>。
随后是一个或多个“补丁块”（hunk），每个块以 @@ 开头（可附带块头信息）。
在补丁块中，每行的前缀含义如下：

关于 [context_before] 与 [context_after] 的规则：
- 默认在每个变更的上下方各展示 3 行代码。若两次变更间隔不超过 3 行，第二个变更的 [context_before] 不要重复上一处的 [context_after]。
- 如果 3 行上下文不足以唯一定位代码片段，可用 @@ 指明所属的类或函数。例如：
@@ class BaseClass
[3 行前置上下文]
- [old_code]
+ [new_code]
[3 行后置上下文]

- 若某段代码在类或函数中出现多次，即便使用一次 @@ 和 3 行上下文仍无法唯一定位，可以串联多个 @@ 来逐级定位。例如：

@@ class BaseClass
@@ 	 def method():
[3 行前置上下文]
- [old_code]
+ [new_code]
[3 行后置上下文]

完整语法定义如下：
Patch := Begin { FileOp } End
Begin := "*** Begin Patch" NEWLINE
End := "*** End Patch" NEWLINE
FileOp := AddFile | DeleteFile | UpdateFile
AddFile := "*** Add File: " path NEWLINE { "+" line NEWLINE }
DeleteFile := "*** Delete File: " path NEWLINE
UpdateFile := "*** Update File: " path NEWLINE [ MoveTo ] { Hunk }
MoveTo := "*** Move to: " newPath NEWLINE
Hunk := "@@" [ header ] NEWLINE { HunkLine } [ "*** End of File" NEWLINE ]
HunkLine := (" " | "-" | "+") text NEWLINE

一个补丁可同时包含多个操作：

*** Begin Patch
*** Add File: hello.txt
+Hello world
*** Update File: src/app.py
*** Move to: src/main.py
@@ def greet():
-print("Hi")
+print("Hello, world!")
*** Delete File: obsolete.txt
*** End Patch

务必牢记：

- 必须使用头部表明动作为 Add/Delete/Update。
- 即便创建新文件，也要在新增行前加 `+`。
- 文件路径必须是相对路径，绝不能使用绝对路径。

调用示例如下：

```
shell {"command":["apply_patch","*** Begin Patch\n*** Add File: hello.txt\n+Hello, world!\n*** End Patch\n"]}
```
