" Vim syntax file
" Language: Raoul
" Inspired by: https://gitlab.com/tsoding/porth/-/raw/master/editor/porth.vim

" Usage Instructions
" Put this file in .vim/syntax/raoul.vim
" and add in your .vimrc file the next line:
" autocmd BufRead,BufNewFile *.ra set filetype=raoul

if exists("b:current_syntax")
  finish
endif

set iskeyword=a-z,A-Z

" Language keywords
syntax keyword raoulKeywords AND OR NOT bool float int string void func if else print while for to global true false return

" Comments
syntax region raoulCommentLine start="//" end="$"

" String literals
syntax region raoulString start=/\v"/ end=/\v"/

" StringTwo literals
syntax region raoulStringTwo start=/\v'/ end=/\v'/

" Number literals
syntax match raoulNumber '\v-?\d+(\.\d+)?'

" Type names the compiler recognizes
syntax keyword raoulTypeNames int float bool string void

" Set highlights
highlight default link raoulKeywords Keyword
highlight default link raoulCommentLine Comment
highlight default link raoulString String
highlight default link raoulStringTwo String
highlight default link raoulNumber Number
highlight default link raoulTypeNames Type

let b:current_syntax = "raoul"
