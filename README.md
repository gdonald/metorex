# Metorex Programming Language

**METOREX** (**M**eta **O**bject **R**untime **E**xecution) is a programming language that combines the expressiveness of Ruby with the performance and safety of Rust. It features a unique **Code-as-Object** meta-programming system that exposes the AST as first-class runtime objects, enabling powerful DSL construction and runtime code manipulation.

⚠️ &nbsp;It's still very early in development.

🙂 &nbsp;[PRs](https://github.com/gdonald/metorex/pulls) and [new](https://github.com/gdonald/metorex/issues/new) [issues](https://github.com/gdonald/metorex/issues) are welcome.

###

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/gdonald/metorex/blob/main/LICENSE) [![CI](https://github.com/gdonald/metorex/workflows/CI/badge.svg)](https://github.com/gdonald/metorex/actions) [![codecov](https://codecov.io/gh/gdonald/metorex/graph/badge.svg?token=GQ4LA1VMRE)](https://codecov.io/gh/gdonald/metorex)

## Project Status

METOREX is currently in **active development** following a 4-phase roadmap:

- **Phase 1 (MVP)**: AST Interpreter with meta-programming core - *In Progress*
- **Phase 2**: Bytecode VM with reflection maturity - *Planned*
- **Phase 3**: Optimization, concurrency, and production features - *Planned*
- **Phase 4**: Advanced features (macros, WebAssembly, functional programming) - *Planned*

See [ROADMAP.md](ROADMAP.md) for detailed implementation plans.

## Key Features

### Core Language Features
- **Exception Handling**: Full begin/rescue/ensure with exception hierarchies and stack traces
- **Pattern Matching**: Powerful pattern matching with destructuring and guards
- **Built-in Testing**: Integrated test framework with assertions and test discovery
- **Traits/Interfaces**: Flexible polymorphism through trait system
- **Optional Type System**: Gradual typing with type inference for performance and safety
- **Advanced Collections**: Set, Deque, PriorityQueue, TreeMap, and immutable structures
- **Struct**: `Struct.new` builds member classes with accessors, `[]`/`[]=`, `to_a`, `to_h`, `each`, `each_pair`, `dig`, `values_at`, and value equality, with `keyword_init:` and a class-body block
- **Struct members and Enumerable**: a member accessor wins over Struct's own `length`, `size`, `members`, and `to_a`, member values stay out of `instance_variables`, `each`/`each_pair`/`select`/`filter` answer an Enumerator without a block, and Struct mixes in Enumerable
- **Struct pattern matching and conversion**: `deconstruct_keys` takes member names, positions, and `to_int` keys, `to_h` accepts a block that returns `[key, value]` pairs, `values_at` takes Ranges, `keyword_init?` reports how the class was built, and writers raise FrozenError on a frozen struct
- **Enumerable**: a class that defines `each` and includes Enumerable answers `to_a`, `entries`, `map`, `select`, `filter`, `find_all`, `reject`, `partition`, `group_by`, `sort`, `sort_by`, `min`, `max`, `minmax`, `min_by`, `max_by`, `minmax_by`, `include?`, `member?`, `take`, `first`, `each_with_index`, `each_entry`, `reverse_each`, `filter_map`, `compact`, `inject`, and the rest of the walk
- **Enumerable argument forms**: `min(n)` and `max(n)` answer the n smallest or largest, `min_by(n)` and `max_by(n)` do the same by the block's value, `find` and `detect` take an ifnone callable, and `take` coerces its count through `to_int`
- **Enumerable enumerators**: a method called without its block answers an Enumerator whose `size` is the receiver's own, and an `each` that yields several values at once hands the block one packed array
- **Non-local return**: `return` inside a block leaves the method that wrote the block, however many yielding methods it travels through first
- **Splatted super arguments**: `super(*list)` spreads the array across the parent's parameters, and mixes with plain arguments as `super(0, *rest)`
- **Enumerable predicates**: `all?`, `any?`, `none?`, `one?`, `count`, and `find_index` take a pattern matched with `===`, which stands in for a block and warns that the block went unused, and more than one argument is an ArgumentError
- **Enumerable batching and counting**: `each_slice` and `each_cons` coerce their width through `to_int` and report the batch count on the Enumerator they answer, `tally` counts into a Hash given as an argument, `zip` reaches an argument through `to_ary` or `to_enum`, and `flat_map` flattens one level through `to_ary`
- **Enumerable counts and sums**: `take`, `drop`, and `first` coerce a count through `to_int` and raise TypeError, ArgumentError, or RangeError for one they cannot use, `find` and `detect` take an ifnone callable, and `sum` adds floats with Kahan-Babuska compensation
- **break through a walk**: `break` in a block leaves the method the block was handed, so `take_while { break :stopped }` answers `:stopped` even when the walk runs through several yielding methods
- **NilClass, TrueClass, and FalseClass**: `nil`, `true`, and `false` answer to their own classes, so `NilClass === nil` and `true.is_a?(TrueClass)` hold
- **Optional parameters before a splat**: `def each(arg = :default, *rest)` binds the default when the call does not reach the parameter
- **Enumerator#each with a block**: walking with a block runs the method the Enumerator was cut from and answers what that method answers
- **Enumerable#inject and #reduce**: fold through a block or through an operator named by a Symbol, a String, or an object answering `to_str`, with the starting value optional, a warning when a block goes unused beside an operator, and an ArgumentError when neither is given
- **inject over a growing Array**: the walk reads one element at a time, so a block that appends to the array reaches what it added
- **Enumerable#grep and #grep_v**: the elements a pattern matches with `===`, or the ones it does not, passed through the block when one is given
- **Enumerator::Lazy**: `lazy` answers a walk that applies `map`, `select`, `filter`, `find_all`, `reject`, `filter_map`, `flat_map`, `collect_concat`, `compact`, `take`, `take_while`, `drop`, `drop_while`, `grep`, `grep_v`, `uniq`, `with_index`, and `zip` one element at a time, so a source with no end still answers `first` and `force`
- **Lazy sizes**: a step that keeps every element reports the source's size, `take` and `drop` report what the count leaves, and a step that decides what to keep reports nil
- **Lazy grouping**: `chunk`, `chunk_while`, `slice_when`, `slice_before`, and `slice_after` cut runs as the walk goes, and the same methods on Enumerable answer an Enumerator over the runs
- **Enumerator::Chain**: `chain` and `+` build a walk over several collections in order, reporting the sum of their sizes and answering nil or Infinity for the first part of such a size
- **Array pickings**: `combination`, `permutation`, `repeated_combination`, and `repeated_permutation` yield each group of a given length and answer the array itself, or an Enumerator reporting how many groups there are
- **Array#product**: the rows drawn one element from each list, reaching an argument through `to_ary` and refusing a number of rows it will not walk
- **Array#bsearch and #bsearch_index**: halve a sorted array rather than walking it, reading true and false from the block or a number saying which way to go
- **Set#classify and #divide**: group the elements under what a block answers, where a block of two parameters instead groups the elements it relates
- **Set#flatten and #flatten!**: open up every set held inside, raising ArgumentError for a set that reaches itself
- **Hash#to_proc**: a lambda of one parameter that reads a key out of the hash, so `[:a, :b].map(&hash)` looks each one up
- **Enumerator#with_object**: walks with a second value handed to the block each time, answering that value when the walk is done
- **A lazy walk pulled one at a time**: `next`, `peek`, and `rewind` run the walk no further than what was asked for, and `String#each_char` without a block answers an Enumerator
- **Data**: `Data.define` builds a value class whose members are read-only, with `new` and `[]` taking positional or keyword arguments, `with` for a copy carrying changes, plus `members`, `to_h`, `deconstruct`, `deconstruct_keys`, `==`, `eql?`, `hash`, and an `inspect` that shows a value reaching itself as the class alone
- **Method#parameters and #arity**: each parameter reports its kind (`:req`, `:opt`, `:rest`, `:keyreq`, `:key`, `:keyrest`, `:block`), and the arity counts the required ones, turning negative where a call may pass more or fewer
- **Proc#parameters and #arity**: a lambda counts the way a method does, while a proc reports its positional parameters as optional and only a splat leaves its count open
- **Method and UnboundMethod rendering**: `inspect` and `to_s` name the receiver's class, the module the method came from when that differs, the parameters, and the file and line
- **Method#super_method**: the next definition of the name above the module the method was found in, following an alias back to what it was cut from
- **UnboundMethod#bind_call**: binds and calls in one step, without an intermediate Method object
- **A class defining `self.new`**: builds its instances that way rather than through allocate-and-initialize
- **`super` with keyword arguments**: the argument list reads the same as any other call's, so keywords, splats, and a block argument all reach the parent method
- **`super` with a block**: `super { ... }` and `super(a) do ... end` both hand the parent method a block of their own, in place of the one the caller supplied. A bare `super` still forwards the enclosing method's arguments
- **`def !` and `def ~`**: an object may name what the operator written in front of it does, and `!obj` reaches the method rather than answering whether the object is false
- **A superclass named from the top level**: `class C < ::Parent` starts the name at the top level rather than inside the module being opened, whether it names a class or builds one, as `class Pair < ::Struct.new(:left, :right)` does
- **A constant bound inside a conditional**: a class assigned to a constant inside an `if` in a module body still reports the namespaced name
- **Time**: a point in time held as a whole number of seconds since the epoch plus an exact fraction, with `now`, `at`, `new`, `utc`/`gm`, and `local`/`mktime` building one
- **Time calendar fields**: `year`, `month`/`mon`, `day`/`mday`, `hour`, `min`, `sec`, `wday`, `yday`, `zone`, `utc_offset`, `dst?`, and the `monday?` through `sunday?` questions, read in UTC, in a fixed offset, or in the zone the operating system holds
- **Time subseconds**: `subsec`, `usec`, and `nsec` keep the fraction exact, so `Time.at(Rational(3, 2)).subsec` answers `(1/2)` and `Time.at(10.75).subsec` answers `(3/4)`
- **Time rendering**: `to_s`, `inspect`, `asctime`/`ctime`, `strftime` (including `%L`, `%N`, `%z`, and `%:z`), `iso8601`/`xmlschema`, and `to_a`
- **Time arithmetic**: `+` and `-` shift by an exact number of seconds, subtracting two times answers a Float, and `<=>`, `==`, `eql?`, `hash`, `round`, `floor`, and `ceil` all read the exact value
- **Time zones**: `utc`/`gmtime`, `getutc`/`getgm`, `localtime`, and `getlocal` move between UTC, a fixed offset given as seconds or as `"+05:00"`, and the local zone
- **ENV writes reach the process environment**: setting `ENV["TZ"]` changes what a local time reads, since the C library sees the same environment
- **A control-flow form read for its value**: `case`, `if`, `unless`, `while`, `until`, and `begin` may be chained onto, so `case x when 1 then :a end.to_s` reads the way it does in Ruby
- **`then` before a newline**: `if cond then` may hold its body on the following line
- **An empty pair of parentheses**: `()` answers nil, which `[0, (), 2]` and `{() => ()}` rely on
- **A group of statements**: `(a; b; c)` answers the last one
- **Percent literals**: `%W` and `%I` fill in their `#{}` parts, and `%q` reads none
- **A chain with the dot leading the line**: newlines and comments may sit between a call and the dot that continues it
- **Safe navigation**: `a&.b` answers nil for a nil receiver without running the method
- **Quoted symbols in `alias`**: `alias :'new' :'old'` names either method that way
- **Libraries metorex carries**: `require` finds `base64`, `shellwords`, `abbrev`, `singleton`, `observer`, `securerandom`, `digest`, `delegate`, `weakref`, `logger`, `monitor`, `tmpdir`, `tempfile`, `bigdecimal`, `zlib`, `erb`, `open3`, `syslog`, `openssl`, `cgi`, `socket`, `net/http`, `net/ftp`, `English`, `io/nonblock`, `yaml`, `rbconfig`, `rbconfig/sizeof`, `resolv`, `optparse`, `rubygems`, `random/formatter`, and `stringio` without a directory on the load path
- **Percent literals with any delimiter**: `%!text!`, `%@text@`, and `%_text_` all read as strings, they fill in their `#{}` parts, and `%x(...)` runs its text as a command. A `%` that follows a value still divides
- **`not` with parentheses**: `not(x)` takes what the parentheses hold, so a call may be chained onto the answer
- **A group with a trailing modifier**: `(123 if true)` and `(count += 1 until done)` answer what the modifier left
- **`Object#methods`**: leaves out the private methods, which are not the ones an object answers to from outside
- **Code named after where it was written**: `__FILE__` inside `eval` reports `(eval at <file>:<line>)`
- **StringIO**: a string read and written the way a file is. Reading with `getc`, `getbyte`, `readchar`, `readbyte`, `gets`, `readline`, `readlines`, `each_line`, `each_char`, `each_byte`, `each_codepoint`, and `ungetc`. Writing with `write`, `print`, `printf`, `puts`, `putc`, and `<<`, padding with NUL when the position sits past the end. Moving with `pos`, `seek`, `rewind`, and `lineno`. Reshaping with `truncate`, `string=`, and `reopen`. Modes decide which sides are open, `close_read` and `close_write` close them apart, and `inspect` shows the class and address alone
- **`String#index` and `#rindex`**: where a substring or pattern first or last sits, counted in characters, with an optional offset
- **Octal escapes**: `"\000"` reads up to three digits as one character
- **An argument may assign**: `StringIO.new(text = "hello")` binds `text` and passes what it assigned
- **`io/console`**: `require` finds it, adding `getch` and `getpass` to StringIO
- **StringScanner**: `require 'strscan'` gives a cursor over a string. `scan`, `check`, `match?`, and `skip` match where the cursor stands, `scan_until`, `check_until`, `skip_until`, `exist?`, `search_full`, and `scan_full` search ahead, and `matched`, `pre_match`, `post_match`, `[]`, `captures`, `named_captures`, and `values_at` report what the last match found. `pos`, `charpos`, `rest`, `eos?`, `bol?`, `reset`, `terminate`, and `unscan` move the cursor, and `getch`, `peek`, `peek_byte`, and `scan_byte` read one piece at a time
- **OpenStruct**: `require 'ostruct'` gives an object whose fields are decided as they are set, with `[]`, `[]=`, `delete_field`, `dig`, `each_pair`, `to_h`, `==`, `marshal_dump`, and an `inspect` that shows a field reaching back to the object as the class alone. A frozen one may be read but not written
- **Prime**: `require 'prime'` gives `Prime.prime?`, `Prime.each` (lazy without a block, so `next` and `rewind` walk it), `Prime.first`, `Prime.prime_division`, and `Prime.int_from_prime_division`, plus `Integer#prime?`, `Integer#prime_division`, `Integer.from_prime_division`, and `Integer.each_prime`
- **Matrix and Vector**: `require 'matrix'` gives rectangular arrays of numbers and the arithmetic over them. Matrices are built with `[]`, `rows`, `columns`, `build`, `diagonal`, `scalar`, `identity`, `zero`, `row_vector`, `column_vector`, and `empty`, and answer `row`, `column`, `[]`, `transpose`, `+`, `-`, `*`, `/`, `**`, `determinant`, `trace`, `rank`, `inverse`, `minor`, `first_minor`, `cofactor`, `collect`, `each`, `each_with_index`, and the shape questions from `square?` through `unitary?`. Vectors answer `+`, `-`, `*`, `inner_product`, `cross_product`, `magnitude`, `normalize`, `each2`, `covector`, and `angle_with`
- **ObjectSpace::WeakMap and ObjectSpace::WeakKeyMap**: two maps that hold their entries only as long as something else does, written here as ordinary maps since metorex frees an object when the last reference to it goes. The first is keyed by identity and answers `[]`, `[]=`, `delete`, `key?`, `member?`, `key`, `size`, `length`, `keys`, `values`, `each`, `each_pair`, `each_key`, and `each_value`, and includes Enumerable. The second is keyed by value, adds `getkey` and `clear`, and refuses a number, a symbol, or one of the three singletons as a key
- **ObjectSpace.memsize_of**: `require 'objspace'` reports how much an object holds, counted from its instance variables and its contents. A name ObjectSpace keeps no account of still answers nil, and a name the library adds now wins over that
- **Message digests**: `require 'digest'` gives `Digest::MD5`, `Digest::SHA1`, `Digest::SHA256`, `Digest::SHA384`, `Digest::SHA512`, and `Digest::SHA2`, which picks one of the last three by bit length. Each answers `digest`, `hexdigest`, `base64digest`, and the `!` forms of all three, both on the class and on an object that takes its message through `update` or `<<`. An object also answers `reset`, `new`, `file`, `length`, `size`, `digest_length`, `block_length`, `to_s`, `inspect`, and an `==` that compares against another digest or against the text of a hexdigest. `Digest.hexencode` names bytes in hex and `Digest.bubblebabble` names them in syllables
- **Naming the parts of a path**: `File.basename` takes the last part, with a named suffix or `".*"` taken off the end, `File.extname` answers the extension a name carries, and `File.split` gives the directory and the name as a pair. Each takes `to_path` or `to_str` from an object that answers one
- **Reading a directory**: `Dir.empty?` says whether one holds anything, `Dir.home` says where a user's files live, `Dir.foreach` and `Dir.each_child` walk the names one at a time or hand back an Enumerator, and `Dir.entries` and `Dir.children` answer them all at once. An `encoding:` keyword tags the names, and `Encoding.default_internal` decides when none is given. `Dir.delete`, `Dir.rmdir`, and `Dir.unlink` remove an empty directory, reporting `Errno::ENOTEMPTY`, `Errno::ENOTDIR`, or `Errno::ENOENT` for one they cannot
- **`File.binread` and `File.binwrite`**: the bytes a file holds, one to a character, so a file that is not text reads back and writes out unchanged
- **Character sets a String reads**: `count`, `delete`, `squeeze`, `tr`, and `tr_s` take the set syntax `tr` writes, so `a-z` is a range, a leading `^` means every character but those, and a descending range is an ArgumentError. `chop`, `delete_prefix`, `delete_suffix`, `partition`, `rpartition`, `intern`, `sum`, `casecmp`, `casecmp?`, `center`, `dedup`, `scrub`, `undump`, `grapheme_clusters`, and `each_grapheme_cluster` read and cut a string without changing it, `upto` walks from one string to another and steps by code point when both are a single character, and `String.try_convert` takes `to_str` from an object that answers one
- **Network addresses**: `require 'socket'` gives `Addrinfo`, which names one endpoint. `Addrinfo.tcp`, `.udp`, `.ip`, and `.unix` build one, `ip_address`, `ip_port`, `ip_unpack`, `afamily`, `pfamily`, `socktype`, and `protocol` report its parts, and `to_s`, `inspect`, `inspect_sockaddr`, and `to_sockaddr` write it back out. The predicates from `ipv4_loopback?` through `ipv6_v4mapped?` say what kind of address it is, and `ipv6_to_ipv4` unwraps the IPv4 address an IPv6 one carries. `Socket::Constants` names the families, socket kinds, and protocols, `sockaddr_in` and `sockaddr_un` write the structs the operating system takes, and `Socket.getaddrinfo` and `gethostname` ask the system
- **TCP sockets**: `TCPServer` listens on a name and a port, `#accept` takes the next connection made to it, and `TCPSocket` reaches one that something is listening on. Both answer `addr`, `close`, and `closed?`, and a connection answers `read`, `write`, `puts`, `peeraddr`, and `remote_address`
- **HTTP requests and responses**: `require 'net/http'` gives `Net::HTTPHeader`, which reads and writes header entries under case-insensitive names. `[]`, `[]=`, `add_field`, `get_fields`, `fetch`, `delete`, `key?`, `to_hash`, and `size` work the entries, `each_header`, `each_name`, `each_value`, `each_capitalized`, and `each_capitalized_name` walk them, and `content_type`, `main_type`, `sub_type`, `type_params`, `content_length`, `content_range`, `range`, `range_length`, `chunked?`, `set_form_data`, `basic_auth`, and `proxy_basic_auth` read and write the ones HTTP names. `Net::HTTPGenericRequest` and the `Net::HTTP::Get` through `Net::HTTP::Unlock` subclasses carry a request and write it onto a socket with `exec`, taking a body from `body=` or a stream from `body_stream=`. `Net::HTTPResponse.read_new` reads a status line and headers off a `Net::BufferedIO` and answers the class the code names, from `Net::HTTPOK` through `Net::HTTPNetworkAuthenticationRequired`, with `code`, `message`, `http_version`, `reading_body`, `body`, `value`, and `error!`. `Net::HTTP` itself opens a connection with `start`, and `get`, `head`, `post`, `put`, `patch`, `delete`, `options`, `trace`, and `request` send one
- **FTP sessions**: `require 'net/ftp'` gives `Net::FTP`. `new` and `open` build a session, taking `passive`, `debug_mode`, `resume`, `open_timeout`, `read_timeout`, `ssl_handshake_timeout`, and `port` as options or as the older positional arguments, and `Net::FTP.default_passive` sets what a session starts in. `connect` and `login` reach a server, `sendcmd`, `voidcmd`, `getresp`, and `voidresp` carry commands and replies, `pwd`, `getdir`, `chdir`, `mkdir`, `rmdir`, `delete`, `rename`, `size`, `mdtm`, `status`, `system`, `site`, `noop`, `help`, `abort`, and `quit` name the operations, and `close` and `closed?` end it. `Net::FTPError` and the `FTPReplyError`, `FTPTempError`, `FTPPermError`, and `FTPProtoError` under it name what the server refused
- **Socket settings**: `Socket::Option` names the family, the level, and the option a setting belongs to, taking each as a symbol, a string, or a number and refusing an unknown one with a SocketError. `Socket::Option.int`, `.bool`, and `.linger` build one, `int`, `bool`, `linger`, `unpack`, and `data` read it back, and `inspect` renders a linger setting the way Ruby does. Every socket answers `binmode?`, `nonblock?`, `close_on_exec?`, and `autoclose?`, `UNIXSocket.socketpair` gives two ends already joined, and `Socket.udp_server_recv` hands each datagram to a block with a `Socket::UDPSource` saying where it came from
- **YAML**: `require 'yaml'` gives `Psych` under both names. `load`, `unsafe_load`, `load_file`, and `load_stream` read scalars, sequences, mappings, flow collections, multi-document streams, and complex keys, naming symbols, numbers, dates, and timestamps as they go, and refusing a malformed mapping with `Psych::SyntaxError`. `dump`, `dump_stream`, and `Object#to_yaml` write them back, tagging a class, a module, a Struct, a Regexp, a Range, an exception, and an object's own fields the way Ruby tags them. `parse` and `parse_file` answer a `Psych::Nodes::Document`
- **Build settings**: `require 'rbconfig'` gives `RbConfig::CONFIG` naming the version, the host, the extensions, and the tools the interpreter was built with, and `RbConfig::TOPDIR` is nil because metorex is one binary rather than an installed tree. `require 'rbconfig/sizeof'` adds `RbConfig::SIZEOF` and `RbConfig::LIMITS`
- **Name lookup**: `require 'resolv'` gives `Resolv` and `Resolv::Hosts`, reading a hosts file for `getaddress`, `getaddresses`, `getname`, and `getnames`, and raising `Resolv::ResolvError` for a name nothing answers to
- **Command-line options**: `require 'optparse'` gives `OptionParser`. `on` describes an option by its short and long names, taking `--[no-]name` for one that negates and `--name VALUE` for one that takes a value, and `parse`, `parse!`, `order`, and `order!` read them out of an argument list, storing what they find in the hash an `into:` keyword names
- **Syslog**: `require 'syslog'` gives the system log. `open`, `close`, `reopen`, and `opened?` work it, `ident`, `options`, `facility`, and `mask` report how it was opened, and `debug`, `info`, `notice`, `warning`, `err`, `crit`, `alert`, `emerg`, and `log` write at their severity. `Syslog::Constants` names the severities, facilities, and options, with `LOG_MASK` and `LOG_UPTO` building a mask. A message goes through the C library, and under `LOG_PERROR` it also reaches `$stderr`
- **OpenSSL digests and keys**: `require 'openssl'` gives `OpenSSL::Digest`, named the way OpenSSL names one and reaching the same algorithms `digest` carries; `OpenSSL::HMAC` for a digest keyed with a secret; `OpenSSL::KDF.pbkdf2_hmac` for a key derived from a password; `OpenSSL::Random` for bytes nobody can guess; and `OpenSSL.secure_compare` and `fixed_length_secure_compare`, which take the same time however two strings differ
- **CGI escaping**: `require 'cgi/escape'` gives `CGI.escape` and `unescape` for a form value, `escapeURIComponent` and `unescapeURIComponent` for a piece of a URL, `escapeHTML` and `unescapeHTML` for the five characters a page reads as markup, and `escapeElement` and `unescapeElement` for the tags of named elements alone
- **Zlib**: `require 'zlib'` gives the two checksums a compressed stream keeps and the encoding underneath it. `Zlib.crc32`, `Zlib.adler32`, and `Zlib.crc_table` answer the values every implementation agrees on, and `inflate`, `deflate`, `gzip`, and `gunzip` read and write the streams. A stream any other zlib wrote reads back here, since the decoder handles stored, fixed, and dynamic blocks alike; what metorex writes uses stored blocks, which every decoder reads. `Zlib::Inflate`, `Deflate`, `GzipReader`, and `GzipWriter` carry the same over a stream, with `GzipReader` walking what it holds a line, a character, or a paragraph at a time
- **ERB**: `require 'erb'` gives templates with Ruby in them. `<%= %>` prints what it names, `<% %>` runs code, and `<%# %>` is a comment. `result`, `result_with_hash`, and `run` render one, `src` shows the Ruby it compiled to, `def_method`, `def_class`, and `def_module` write it onto a class as a method, and `ERB::Util` adds `html_escape` and `url_encode` under their short names too
- **Open3**: `require 'open3'` runs a command and hands back what it wrote. `capture2` answers its output and status, `capture2e` joins its error stream onto the output, and `capture3` keeps the two apart. `popen2`, `popen2e`, `popen3`, and the `pipeline` family hand the streams over instead
- **BigDecimal**: `require 'bigdecimal'` gives decimal numbers with as many digits as they are given. A value is a sign, its significant digits, and the power of ten they sit against, which `split`, `exponent`, `precision`, and `sign` report. Arithmetic covers `+`, `-`, `*`, `/`, `div`, `quo`, `%`, `modulo`, `remainder`, `divmod`, and `**`, each carrying the digits asked for, and `add`, `sub`, and `mult` take a count of their own. `round`, `ceil`, `floor`, `truncate`, `fix`, and `frac` cut at a named place under any of the seven rounding rules, spelled either by name or by constant. `sqrt` takes a count of digits, `to_s` writes the number in exponent or full form with optional grouping, and `NaN` and the two infinities answer `nan?`, `infinite?`, and `finite?`. `BigMath` adds `PI`, `E`, and `sqrt`, and `bigdecimal/util` adds `to_d` to Integer, Float, String, Rational, and nil
- **Delegation**: `require 'delegate'` gives `Delegator`, `SimpleDelegator`, and `DelegateClass(Klass)`. A delegator passes on whatever the object behind it answers to, settles `==`, `!=`, `equal?`, and `eql?` against itself before asking, and reports the object's methods alongside its own. `__getobj__` and `__setobj__` say which object is standing behind it
- **WeakRef**: `require 'weakref'` gives a reference that delegates to an object and answers `weakref_alive?`. Metorex frees an object when the last reference to it goes, so the reference holds it through ObjectSpace's weak map
- **Logger**: `require 'logger'` gives a log with a severity on each message. `debug`, `info`, `warn`, `error`, `fatal`, and `unknown` write at their own level, `add` and `log` take one, and `level` decides which of them reach the device. A level may be named or numbered. `Logger::LogDevice` writes to an IO or to a file it opens by name, rotating that file when it is asked to keep more than one, and `Logger::Formatter` decides how a line reads
- **Monitor**: `require 'monitor'` gives `Monitor` and `MonitorMixin`, a lock a thread may take more than once. `enter`, `exit`, `try_enter`, `synchronize`, `mon_locked?`, and `mon_owned?` work it, `new_cond` makes a `MonitorMixin::ConditionVariable` to wait on, and leaving a monitor nobody is holding raises `ThreadError`
- **Tempfile**: `require 'tempfile'` gives a file with a name nobody else is using. `Tempfile.new`, `.open`, and `.create` make one, `path` names it, `close` and `close!` finish with it, and `unlink` removes it. `Dir.tmpdir` names the directory they go in
- **`File.rename` and `File.chmod`**: move a name onto another, and set the permissions on the names given
- **A Singleton is reached through `instance` alone**: both `new` and `allocate` are refused on a class that includes it
- **Etc**: `require 'etc'` reads the password and group databases and what the system reports about itself. `getpwuid`, `getpwnam`, `getpwent`, `setpwent`, `endpwent`, and `passwd` walk the accounts, `getgrgid`, `getgrnam`, `getgrent`, `setgrent`, `endgrent`, and `group` walk the groups, and both hand back `Etc::Passwd` and `Etc::Group` structs. `uname`, `nprocessors`, `sysconf`, `confstr`, `getlogin`, `sysconfdir`, and `systmpdir` answer the rest, with the `SC_` and `CS_PATH` constants the C library knows
- **Coverage**: `require 'coverage'` gives `supported?`, `running?`, `start`, `result`, and `peek_result`. No measurement is recorded yet, so every mode reports unsupported and a result is empty
- **A `key => value` pair without parentheses**: `held.update "a" => 1` and `send @method, Object.new => "0"` gather their pairs into a Hash the same way a call written with parentheses does
- **An unknown pack directive raises ArgumentError**: `[1].pack('%')` reports `unknown pack directive '%' in '%'`
- **Process identity and scheduling**: `Process.getpgrp`, `setpgrp`, `getpgid`, `setpgid`, `getsid`, and `setsid` name the group and session a process belongs to, `getpriority` and `setpriority` read and write how much of the processor it is given, and `initgroups` and `groups=` set the supplementary groups it runs with. `Process.times` answers a `Process::Tms`, `setproctitle` renames the process in a listing, and `Process._fork` reports that forking is not carried here. `Process::GID`, `Process::UID`, and `Process::Sys` name the same ids Process itself does, and the WNOHANG, PRIO_, RLIMIT_, and RLIM_ constants carry the numbers the operating system names its settings by
- **Signals reach the operating system**: `Process.kill` sends the signal itself rather than only raising, so a process that kills itself ends with that signal and one sent to a process that is not there reports `Errno::ESRCH`. A `SignalException` nothing catches ends the program the way the signal would have, which is what a wait reports back as `Process::Status#termsig`
- **FileTest**: the questions File answers about a path, gathered in a module so they can be asked without naming File. `exist?`, `file?`, `directory?`, `readable?`, `readable_real?`, `writable?`, `writable_real?`, `executable?`, `executable_real?`, `symlink?`, `blockdev?`, `chardev?`, `pipe?`, `socket?`, `setuid?`, `setgid?`, `sticky?`, `owned?`, `grpowned?`, `identical?`, `zero?`, `empty?`, `size`, `size?`, `world_readable?`, and `world_writable?`
- **A path may be named by an object**: `File.exist?`, `file?`, `directory?`, `executable?`, `symlink?`, `size`, `size?`, and `readlink` ask an argument that is not a String for `to_path`, and then for `to_str`, before reaching the filesystem
- **File.birthtime**: answers when a file was made, the same time `File.stat(path).birthtime` reports
- **Process.groups**: the supplementary groups this process belongs to. `grpowned?` counts them, so a file whose group is any of them belongs to this process's group
- **`RUBY_PLATFORM` names an architecture and an operating system**: `arm64-darwin` rather than `macos`, which is the form Ruby reports and the form the spec suite's platform guards read
- **An arithmetic sequence**: `1.step(10, 3)` and `(1..10).step(3)` answer an `Enumerator::ArithmeticSequence` when they are handed no block, and `(1..10) % 3` writes the same thing. It answers `begin`, `end`, `step`, `exclude_end?`, `each`, `first`, `last`, `size`, `hash`, `==`, and an `inspect` that writes it back the way it was asked for. A sequence with no end reports `Float::INFINITY` as its size, and it is built only through those methods, so `new` raises NoMethodError and `allocate` raises TypeError
- **Enumerator::Product**: the Cartesian product of several walks, yielding one array per combination in the order the walks were given. It answers `each`, `to_a`, `size`, `rewind`, `inspect`, and a private `initialize_copy` that refuses a frozen receiver, a different class, and an uninitialized argument. `size` is nil as soon as one walk cannot say how long it is
- **Enumerator::Yielder takes a block**: `Enumerator::Yielder.new { |value| ... }` hands everything to that block. `yield` passes its arguments through and answers what the block answers, `<<` passes one value and answers the yielder, and `to_proc` answers the block itself
- **Rewinding a walk rewinds its source**: `Enumerator#rewind` hands the rewind on to the object the walk was cut from when that object can be rewound
- **A missing method names its receiver the way Ruby does**: `undefined method 'each_entry' for an instance of Object`, and `for nil`, `for true`, `for class Foo`, or `for module Bar` when that is what it was called on
- **A class may answer `allocate` itself**: a `def self.allocate` is what the class hands back, and a subclass inherits it
- **An empty matrix keeps its shape**: `Matrix.columns([[], [], []])` is 0 by 3 and `Matrix.column_vector([])` is 0 by 1, `transpose` swaps the two sizes, `inspect` writes `Matrix.empty(0, 3)`, and two empty matrices of different widths are not equal. Multiplying by an empty matrix answers one as wide as the right side
- **Matrix#minor and Matrix#find_index**: `minor` reads a rectangle given as a start and a count on each side or as two ranges, counts backwards from a negative start, ignores a count past the edge, and answers nil for a start out of range or a negative count. `find_index` reads a value, a block, or one of `:diagonal`, `:off_diagonal`, `:lower`, `:strict_lower`, `:strict_upper`, and `:upper`, and answers an Enumerator when given neither a value nor a block
- **A size given to `Matrix.build` goes through `to_int`**, so anything that converts is accepted and anything else raises TypeError. `Vector#each2` and `Vector#collect2` pair with any sized and indexable object, including a plain Array, and `Vector#inner_product` takes the conjugate of its argument
- **A subscript may be written across several lines**: a newline after the opening bracket carries no meaning, so `Matrix[` may be followed by one row per line before the closing bracket
- **`private_class_method :new`**: a class method the runtime answers can be made private, so a class built only through its named builders refuses `new` and reports false from `respond_to?(:new)`
- **`Enumerator#to_a` collects without answering the collection**: the block it walks with answers nil, so a method that stops on the first truthy block result walks the whole way when it is asked for an Enumerator
- **Date**: `require 'date'` gives a calendar day held as the Julian Day Number it stands for, together with the day the Gregorian calendar takes over from the Julian one. Dates are built with `civil`, `jd`, `ordinal`, `commercial`, `today`, `parse`, `strptime`, `iso8601`, and `rfc3339`, and answer `year`, `month`, `day`, `yday`, `wday`, `cwyear`, `cweek`, `cwday`, `mjd`, `ajd`, `amjd`, `ld`, `leap?`, and a question per weekday. `+`, `-`, `>>`, `<<`, `next_day`, `prev_month`, `succ`, `upto`, `downto`, and `step` move about, `new_start`, `italy`, `england`, `julian`, and `gregorian` say which calendar a date is read on, and `strftime`, `to_s`, `inspect`, `asctime`, `iso8601`, and `rfc3339` write one out. `Date.valid_civil?`, `valid_ordinal?`, `valid_commercial?`, `valid_jd?`, `leap?`, `gregorian_leap?`, and `julian_leap?` answer without raising, and `Date::Infinity` stands for a calendar that never reforms
- **DateTime**: `require 'date'` also gives a calendar day carrying a time of day and an offset from UTC, held as an exact fraction of a day so arithmetic below a millisecond loses nothing. Built with `new`, `civil`, `jd`, `ordinal`, `commercial`, `now`, `parse`, `strptime`, `iso8601`, `rfc3339`, `rfc2822`, and `httpdate`, and answering `hour`, `min`, `sec`, `sec_fraction`, `offset`, `zone`, `new_offset`, `to_date`, `to_time`, and every Date method besides. An hour written as a negative counts back from the next day, and 24 names midnight on the day after
- **The time library**: `require 'time'` adds `Time.parse`, `Time.iso8601`, `Time.xmlschema`, `Time.rfc2822`, and `Time.httpdate`, along with `Time#iso8601`, `Time#xmlschema`, `Time#rfc2822`, and `Time#httpdate`. `Time#to_datetime` and `Time#to_date` cross to the calendar the offset a Time carries puts it on
- **strftime writes the clock**: `%H`, `%I`, `%k`, `%l`, `%M`, `%S`, `%L`, `%N` with a digit count, `%P`, `%p`, `%s`, `%Q`, `%z`, `%:z`, and `%::z` all read the time and offset the receiver holds
- **String interpolation asks for `to_s`**: `"#{object}"` calls the object's own `to_s` rather than writing the default `#<Class:0x...>` form
- **`Float#to_r` is exact**: `0.00001001.to_r` is the fraction the float actually holds, not one truncated partway through
- **A Rational compares by value inside a collection**: a Hash or an Array holding `Rational(0, 1)` equals one holding `0`
- **URI**: `require 'uri'` gives `URI.parse`, `URI.split`, `URI.join`, `URI.extract`, `URI.regexp`, `URI.escape`, `URI.unescape`, the `encode_www_form` and `decode_www_form` pairs, and the `URI(text)` method. The scheme decides the class: `URI::HTTP`, `HTTPS`, `FTP` with its typecode, `LDAP` with its distinguished name and filter fields, `MailTo` with its address and headers, `WS`, `WSS`, `File`, and `Generic` for everything else. A URI answers every component it holds, `merge`, `route_to`, `route_from`, `normalize`, `select`, and `request_uri`, and refuses a component the scheme has no room for
- **CSV**: `require 'csv'` reads comma separated text into rows of fields with `CSV.parse`, `parse_line`, `read`, `readlines`, `foreach`, and `open`, and writes it back with `CSV.generate_line` and `CSV#<<`. An empty field is nil and a quoted empty one is a string, a quoted field may carry the separator, a newline, and its own doubled quotes, `col_sep:` names another separator, and text a reader cannot make sense of raises `CSV::MalformedCSVError` unless `liberal_parsing:` is asked for
- **Pathname**: `require 'pathname'` gives a path as an object, with the naming operations kept apart from the ones that reach the filesystem. `+`, `/`, `join`, `parent`, `basename`, `dirname`, `extname`, `sub`, `sub_ext`, `cleanpath`, `relative_path_from`, `each_filename`, `descend`, and `ascend` work on the name alone, while `exist?`, `file?`, `directory?`, `empty?`, `size`, `read`, `children`, `glob`, and `realpath` ask the filesystem. `Pathname(path)` hands back a Pathname that arrives already made
- **File.size and Dir.entries**: `File.size(path)` and `File.size?(path)` answer how many bytes a file holds, and `Dir.entries(path)` and `Dir.children(path)` list what a directory holds
- **Dir.glob takes flags and a base**: `Dir.glob(pattern, File::FNM_DOTMATCH)` matches names that open with a dot, and `base:` reads the pattern against a directory and answers paths relative to it. The `File::FNM_*` constants are defined
- **An array of objects compares with their own equality**: `[held] == [other]` asks the objects, so two Pathnames naming the same path make their arrays equal
- **An open Dir**: `Dir.new` and `Dir.open` answer a directory that is walked one name at a time, with `read`, `each`, `each_child`, `pos`, `tell`, `seek`, `pos=`, `rewind`, `entries`, `children`, `path`, `to_path`, `close`, and `closed?`. Walking the whole of one leaves the position at the end, the path stands after it is closed, and every other method on a closed one raises IOError. `Dir` includes Enumerable
- **ENV answers a few things its own way**: `to_s` is "ENV", `rehash` is nil, `to_h` and `to_hash` hand back a copy so changing it leaves the environment alone, and `dup` and `clone` are refused the way Ruby refuses them
- **Encoding.find**: names an encoding, with "locale", "external", and "filesystem" answering what those settings hold and "internal" answering `Encoding.default_internal`. `String#force_encoding` answers the string it was given
- **`pretty_inspect`**: every object answers it, which is what `require "pp"` adds in Ruby and what mspec asks for when it reports a failure
- **A symbol is quoted when its name needs it**: `:"a b"` and `:"1x"` are written with quotes, while `:@a`, `:$b`, `:foo?`, `:[]`, and the operator names stand bare
- **An escaped space keeps a percent-list word whole**: `%w[a\ b c]` is two words, not three
- **File::Stat**: `File.stat` and `File.lstat` answer a real File::Stat, reading what the operating system keeps about a file. `dev`, `ino`, `mode`, `nlink`, `uid`, `gid`, `rdev`, `size`, `blksize`, `blocks`, the major and minor pairs, `atime`, `mtime`, `ctime`, `birthtime`, `ftype`, `inspect`, `<=>`, and the questions from `file?` through `world_writable?`. `File` asks the same questions of a path with `ftype`, `zero?`, `size?`, `identical?`, `owned?`, `grpowned?`, `setuid?`, `setgid?`, `sticky?`, `blockdev?`, `chardev?`, `pipe?`, `socket?`, `world_readable?`, and `world_writable?`
- **Links**: `File.symlink`, `File.readlink`, and `File.link` make and read them, and `File.symlink?` answers what the filesystem says rather than always false
- **File.chown, File.lchown, and File.utime**: set who owns a file and the times it reports
- **Dir.entries and Dir.children**: list what a directory holds
- **Process.gid and Process.egid**: answer the group this process runs as
- **`method_missing` is handed what Ruby hands it**: the name arrives as a Symbol and the call's arguments arrive one by one, so `def method_missing(name, path)` reads the path as itself rather than as an array holding it
- **`Integer#to_s` takes a base**: `255.to_s(16)` is "ff"
- **The reported Ruby version is 4.0.1**: `RUBY_VERSION` and `-v` name the release the vendored ruby/spec suite is written for, which is what its own CI runs the specs against. `METOREX_RUBY_VERSION` names another release instead, and the spec suite's version guards follow it, so `RUBY_VERSIONS="4.0.1 3.4.8 3.3.10" scripts/run_ruby_spec.sh` runs the whole queue once per release and reports each
- **A raise inside a Set block carries on as itself**: `Set[1].each { raise ArgumentError }` reaches a `rescue ArgumentError` rather than arriving as a RuntimeError
- **A Set decides equality itself**: `set == other` reaches Set's own `==`, so anything that answers `is_a?(Set)` is compared by the elements it holds
- **GetoptLong**: `require 'getoptlong'` reads command line options the way the GNU getopt_long function does. An option may be written in full or as one letter, one-letter options may be run together, a word may follow an equals sign or stand as the next word, a shortened name is taken when only one option starts with it, and `--` ends the options. `get`, `each`, `terminate`, `terminated?`, `ordering=`, `set_options`, `error_message`, and the `MissingArgument`, `InvalidOption`, `AmbiguousOption`, and `NeedlessArgument` errors are all there
- **IPAddr**: `require 'ipaddr'` gives an IPv4 or IPv6 address held as the number it stands for, with the mask that says how much of it names the network. It reads the short IPv6 form, a dotted quad written inside an IPv6 address, and a mask written as a length or as an address, and answers `to_s`, `to_string`, `inspect`, `family`, `prefix`, `reverse`, `ip6_arpa`, `ip6_int`, `native`, `ipv4_compat`, `ipv4_mapped`, `include?`, `mask`, and the bit operations `&`, `|`, `<<`, `>>`, and `~`
- **Find**: `require 'find'` walks every path beneath a directory, and `Find.prune` leaves the directory it was handed unwalked
- **Timeout**: `require 'timeout'` gives `Timeout.timeout`, which runs a block and answers what it answers
- **Random is the Mersenne Twister**: `Random.new(seed)` draws from MT19937 seeded the way Ruby seeds it, so a seed gives the same numbers here as it does there. `bytes`, `rand`, `random_number`, `seed`, `==`, `Random.new_seed`, `Random.urandom`, and `Random.srand` are all there, and `srand` settles what the seedless `rand` answers as well
- **A Range bounds a draw by its width**: `rand(first..last)` measures `last - first`, so any type that subtracts and adds bounds a draw of its own kind, and a pair of ends that cannot be subtracted raises ArgumentError
- **Encoding::Converter**: names the two encodings a conversion runs between, the steps it takes, and what stands in for a character the destination cannot spell. `source_encoding`, `destination_encoding`, `convpath`, `Encoding::Converter.search_convpath`, `Encoding::Converter.asciicompat_encoding`, `replacement`, `inspect`, and the flag constants are defined
- **A source written in bytes keeps them**: a `# encoding: binary` magic comment makes a run of `\xNN` or `\NNN` escapes name those bytes one by one, while elsewhere bytes that spell a character in UTF-8 read back as that character
- **Byte counts are byte counts**: `bytesize`, `bytes`, and `getbyte` read a binary string one byte to a character, and `StringIO#pos` and `StringScanner#pos` count the bytes before the cursor rather than the characters
- **A path may name itself**: `File.directory?`, `File.symlink`, and the other predicates take anything answering `to_path` or `to_str`, and the predicates take a stream through `to_io`
- **ENV is looked up by text**: `has_key?`, `include?`, `member?`, `key?`, `key`, `delete`, `assoc`, `has_value?`, `value?`, and `rassoc` ask an argument for `to_str`. A name that reads as nothing is refused, and a value that reads as nothing matches nothing
- **A time in UTC says so**: a zone written as "UTC", "Z", or "-00:00" names UTC itself, `Time.now`, `Time.new`, and `Time.at` take it through `in:`, and `Time#zone` answers in US-ASCII
- **A String changes what it holds**: `<<`, `concat`, `replace`, `prepend`, `insert`, `clear`, `setbyte`, `slice!`, and the `!` forms of `sub`, `gsub`, `strip`, `chomp`, `chop`, `squeeze`, `delete`, `tr`, `tr_s`, `upcase`, `downcase`, `capitalize`, `swapcase`, `reverse`, `succ`, and `encode` all change the string every reference to it sees. A method whose name ends in `!` answers nil when it found nothing to change
- **`freeze` and `frozen?` mean what they say**: a String reports itself frozen only after it has been frozen, a change to a frozen one raises FrozenError, `+str` answers one that changes, and `-str` answers one that does not
- **Case mapping takes options**: `:ascii` leaves everything but the ASCII letters alone, `:turkic` maps the dotted and dotless `i` to their own pairs, `:lithuanian` maps the way the full rules do, and `:fold` on `downcase` turns a sharp s into two of them
- **`valid_encoding?` and `ascii_only?`**: a run of bytes tagged as text says whether those bytes spell characters there, and an encoding that spells even the ASCII letters in more than one byte holds nothing ASCII-only
- **`Integer#chr`**: a number under 128 names an ASCII character and one up to 255 names a byte of its own, and a wider one needs an encoding named: `0x3042.chr("UTF-8")`
- **StringIO reads the way IO does**: `read`, `sysread`, and `read_nonblock` fill a buffer handed to them and answer it, `gets` and `readline` set `$_` where `each_line` and `readlines` leave it alone, every line reader takes a separator that reads as a String, a limit that reads as an Integer, and `chomp:`, and `truncate` cuts the buffer itself
- **A Refinement names what it refines**: `refine` answers a Refinement, which reports `target` and `refined_class`
- **A collection carries instance variables**: `instance_variable_set` and `instance_variable_get` work on an Array, a Hash, or a Set, and a new collection built from one does not carry them
- **A lambda counts its arguments wherever it is called**: a lambda handed to `each` as a block is refused the same way a call with the wrong number of arguments is
- **A symbol's characters are ASCII when that is all they are**: `Symbol#encoding`, `#to_s`, `#id2name`, and `#name` answer US-ASCII for a name written in ASCII
- **`File.expand_path` reads a leading `~`**: it names the home directory, and `Kernel#system` takes `out:` and `err:` to say where the child's streams go
- **A left shift is exact**: `1 << 200` keeps its value rather than dropping the high bits
- **The rest of the operator assignments**: `%=`, `**=`, `|=`, `&=`, `^=`, `<<=`, and `>>=` all parse
- **A percent list nests its delimiters**: `%w[a [ b ] c]` reads its brackets as words rather than ending at the first one
- **Setting an ENV name to nil removes it**: `ENV["X"] = nil` makes `ENV.include?("X")` false, the way Ruby's does
- **ARGV is always there**: a program reads or replaces it whether or not a caller handed one over
- **A method may be named for a keyword**: `def self.for` parses, and `URI.for` calls it
- **A call form reaches a method sharing a constant's name**: `URI(text)` calls the method where a bare `URI` names the module, and `Kernel::URI(text)` does the same through a module
- **String#scan**: walks every match of a pattern, answering the matched text or, when the pattern captures, the groups it took
- **String#gsub and #sub take a block**: the block is called with each match and answers what replaces it
- **String#split takes a limit**: a positive one caps the fields, and trailing empty fields are dropped unless the limit is negative
- **String#to_i takes a base**: `"ff".to_i(16)` is 255
- **String#slice and #[] take one argument**: an Integer, a Range, another String, or a pattern
- **A quoted string may run across lines**: a newline inside `"..."` is content rather than the end of the literal
- **`next` ends a block with its value**: `values.map { |held| next 0 if held.nil?; held }` answers 0 for the nil ones
- **A symbol carries no space after its colon**: `flag ? held : "text"` is a ternary rather than a call passing `:"text"`
- **A method may be named for the right shift operator**: `def >>(count)` parses
- **`self.` reaches a private method**: `self.hidden` calls it the way a bare `hidden` does, rather than raising NoMethodError
- **The recursion guard counts per thread**: two virtual machines running side by side no longer add their nesting together
- **A splat spreads across a subscript**: `Held[*values]` passes the values one by one, the same way `Held.[](*values)` does
- **A trailing comma destructures anyway**: `first, = pair` takes the first element the way a fuller target list would
- **An assignment reads as an operand**: `count.should == held += 1`, `4 < held += 1`, and `ready && held = 1` each read the assignment as the right operand, the way Ruby reads them
- **A method name is not a local variable**: `take ['a']` passes the array, where before the name being defined was mistaken for a variable and the brackets read as a subscript
- **A `when` reaches a constant through an included module**: `when Socket::SOCK_DGRAM` matches, where the constant lives on a module the namespace includes
- **`alias` renames a global**: `alias $ERROR_INFO $!` gives one global a second name that reads and writes the first
- **A splat spreads a range**: `[*"a".."z"]` reads the whole range and spreads the values it covers, where before the splat bound to the first value alone
- **A bitwise operator carries a line on**: `held = one |` followed by the next line reads as one expression, the way it does for `+`
- **A number can carry no singleton method**: `def one.name` on an Integer, a Float, or a Symbol raises TypeError, since the same object turns up wherever that value does
- **`Float#round` answers a whole number**: with no argument, or with a count of digits at or left of the point, it answers an Integer the way Ruby's does
- **Two hashes compare by what they hold**: a value that is an object of the program's own is asked with `==` rather than compared as data, which is what `Array#==` already did
- **A splat destructures a multiple assignment**: `first, *rest = values` gives `rest` everything the named targets leave, and the splat may sit anywhere in the list, so `*leading, last`, `head, *middle, tail`, and a bare `*everything` all read the same way. A splat with nothing left for it takes an empty Array
- **`super` from a constructor with nothing above it**: an `initialize` whose ancestors define none reaches Object's, which takes no arguments and answers nil
- **A method may be named for the power operator**: `def **(count)` parses
- **An unknown name reaches `method_missing`**: an attribute assignment with no setter behind it, and `send` of a name with no method behind it, both go there
- **Singleton definition on an expression**: `def (@matcher = Object.new).===(other)` assigns first and defines the method on what the variable holds
- **`__FILE__`**: names the file the code was written in, so a method or block from a required file reports that file rather than the one being run
- **MatchData**: a match answers one, with `[]` by index or group name, `captures`, `named_captures`, `names`, `values_at`, `begin`, `end`, `offset`, `match`, `match_length`, `pre_match`, `post_match`, `to_a`, `deconstruct`, `deconstruct_keys`, and equality by subject, pattern, and positions
- **Named groups**: once a pattern names any group the unnamed ones stop capturing, which is what puts `captures` and `inspect` on the named ones alone
- **Match globals**: `$~`, `$1` through `$9`, `` $` ``, `$'`, `$&`, and `Regexp.last_match` all read the last match, which a pattern that finds nothing clears, and `grep` without a block leaves alone
- **Regexp methods**: `match`, `match?`, `=~`, `===`, `source`, `options`, `casefold?`, `names`, `named_captures`, `to_s`, `inspect`, `==`, and `hash`, with `Regexp.new`, `Regexp.compile`, `Regexp.union`, and `Regexp.last_match`; a pattern reports Regexp as its class
- **String#match**: answers the MatchData for a Regexp or a String pattern, from an optional offset, and hands it to a block when one is given
- **Case equality**: a Range answers `===` the way `cover?` does, and a Regexp reaches an operand's characters through `to_str`
- **Set**: keeps its elements in the order they were added and holds any value with a stable rendering, so a Symbol, an Array, or nil is an element like a number is
- **Set algebra**: `union`, `difference`, `intersection`, and their `|`, `+`, `-`, `&`, `^` spellings, with `subset?`, `superset?`, `proper_subset?`, `proper_superset?`, `disjoint?`, and `intersect?`; an operand may be any Enumerable
- **Set in place**: `<<`, `add?`, `merge`, `subtract`, `replace`, `clear`, `delete_if`, `keep_if`, `select!`, `reject!`, and `map!`, plus `join`, `hash`, `dup`, and an `each` that answers an Enumerator without a block
- **Orphaned return**: `Proc.new { return }` called after its defining method has returned raises a LocalJumpError carrying `exit_value` and `reason`
- **Set membership**: `include?`, `member?`, and `===` ask the element for its `hash` and `eql?`, so two objects that agree on both are the same element; an Integer and a Float of the same value are not
- **Set during iteration**: adding to, merging into, replacing, or clearing a Set while a walk over it is open raises a RuntimeError rather than changing what the walk is reading
- **allocate**: `Array.allocate` and `Hash.allocate` answer an empty one, `Proc.allocate` raises TypeError, and `MatchData.allocate` does not exist
- **Proc method table**: `Proc` reports the methods it answers, so `public_instance_methods` lists `call`, `eql?`, `==`, `arity`, `curry`, and the rest
- **Array#join**: a Symbol element contributes the name it is spelled with, and a nil separator joins with nothing between
- **Hash#replace and #transform_values!**: `replace` answers the receiver even when the new contents are written as keyword arguments, and a `break` out of `transform_values!` keeps what it changed
- **Range#size**: reports a count only when the range steps from an Integer, answers nil for one that steps from a String or Symbol, and raises TypeError for one it cannot step from at all
- **start_with? with a Regexp**: records the match, so `$~`, `$1`, and `Regexp.last_match` read it afterwards
- **Repeated group names**: a pattern may write the same name on more than one group, and the match reports the farthest one under it that matched; a name spelled with multi-byte characters counts in characters
- **Regexp introspection**: `names` lists each written name once and `named_captures` reports the group numbers each was written on, both on the pattern and on the match
- **Regexp equality**: two patterns differing only in the `/n` encoding option are the same pattern, and hash the same
- **sub and gsub record the match**: `$~`, `$1`, and `Regexp.last_match` read it afterwards, whether the pattern was a Regexp or a String
- **Set comparison and identity**: `<=>` reports containment, `delete` answers the set while `delete?` answers nil for an element that was not there, and two names for one native method answer the same Method object
- **Set subclasses**: `class Bag < Set` instances carry their elements and answer Set's methods, which is what lets `to_set(Bag)` build one
- **Set rendering**: a Set inspects as `Set[1, 2]`, and one that reaches itself as `Set[...]`
- **String#to_str, #codepoints, and #each_codepoint**: the implicit conversion and the code point of each character
- **String#to_r**: an underscore between digits is a separator rather than part of the number
- **Enumerable#sum**: an infinity stays one rather than turning into a NaN, since there is no rounding to compensate for
- **Lambda parameters**: a lambda written without parentheses takes defaults and keyword parameters the same as one written with them, so `-> name = "world" { }` and `-> *rest, tag: :none { }` both read as declared
- **Lambda arity**: a lambda takes its arguments the way a method does, refusing a call that gives it the wrong number
- **`**nil`**: a method declares that it takes no keyword arguments at all
- **A method as a block**: `&some_method` hands the method over as the block, keeping its own arity, so `hash.each(&recorder.method(:record))` gets one `[key, value]` pair per entry
- **match with a block**: `String#match`, `Symbol#match`, and `Regexp#match` hand the MatchData to a block and answer what the block answers; `match?` takes the offset to start at
- **Symbol has no constructor**: `Symbol.new` and `Symbol.allocate` are undefined, the way Ruby leaves them
- **Nested Sets**: a Set holding Sets compares by what those hold, so the order they were added in does not matter
- **Kernel#trap**: Kernel's name for `Signal.trap`, listed among its private instance methods
- **String bytes**: `bytesize`, `bytes`, `each_byte`, and `getbyte` read a String as the bytes it is made of, alongside `chr`, `ascii_only?`, and `valid_encoding?`
- **String#hex and #oct**: read a number off the front of the text, `hex` honoring only the `0x` prefix and `oct` honoring `0x`, `0b`, `0o`, and `0d`, with an underscore between digits treated as a separator
- **Kernel Conversion Functions**: `Hash()`, `Integer()`, `Rational()`, and `String()`, with coercion through `to_hash` / `to_int` / `to_i` / `to_r` / `to_s` and `exception: false`
- **Rational Numbers**: the `5r` literal suffix, arithmetic and ordering against Integer, Float, and Rational, `to_r` on String, Integer, and Float, and results always in lowest terms and frozen
- **Numeric Literals**: decimal, `0x`/`0b`/`0o`/`0d` radix prefixes, bare-leading-zero octal, scientific notation, `_` digit separators, and the `r` rational suffix
- **String Subclasses**: `class Name < String` instances carry their characters and answer String's methods, comparing equal to a String with the same content
- **Runtime Class System**: Classes support inheritance, runtime method definition, instance variables, and class-level state
- **File Loading**: `require_relative` with extension auto-detection, deduplication, circular dependency handling, and shared scope

### Meta-Programming (Core Innovation)
- **Code-as-Object**: AST nodes are first-class objects manipulable at runtime
- **Runtime Method Definition**: `define_method` for dynamic behavior
- **Method Missing Hook**: `method_missing` intercepts calls to undefined methods with method name and arguments
- **Runtime Class Modification**: `remove_method`, `undef_method`, `alias_method`, `module_function`, `class_variable_set`, `class_variable_get`, `class_variable_defined?`, `class_variables` for dynamic class/module manipulation
- **Reflection and Introspection**: `class`, `instance_of?`, `is_a?`, `itself`, `respond_to?`, `methods`, `send`, `instance_variables`, `instance_variable_get`, `instance_variable_set`, `local_variables`, `__method__`, `__callee__`
- **AST Manipulation**: `eval` for runtime code execution, `parse` for AST inspection, runtime code generation via string evaluation
- **Block Execution**: Blocks are objects with `.call()` method; trailing `do...end` and `{...}` blocks captured implicitly via `&block` parameter with `block_given?` support
- **DSL Construction**: Build domain-specific languages naturally

### Standard Library
- **Networking**: HTTP client/server, WebSocket, TCP/UDP, TLS/SSL
- **Serialization**: JSON, XML, YAML, CSV, MessagePack
- **Cryptography**: Hashing, encryption, secure random, certificates
- **Concurrency**: OS threads, fibers, async/await, channels, atomics
- **Advanced Math**: Complex numbers, arbitrary precision, statistics

### Developer Experience
- **Documentation System**: Doc comments with automatic HTML generation
- **Debugger**: Full debugging with breakpoints and inspection
- **LSP Support**: Language Server Protocol for IDE integration
- **Build System**: Incremental compilation, profiles, and optimization
- **Linter & Formatter**: Code quality and style enforcement

## Core Philosophy and Identity

| Element                 | Description                                                                                                                       |
| :---------------------- | :-------------------------------------------------------------------------------------------------------------------------------- |
| **Foundation Language** | **Rust** (for VM safety and speed)                                                                                                |
| **Syntax Heritage**     | **Ruby** (block structure, optional parentheses)                                                                                  |
| **Primary Paradigms**   | **Full Object-Oriented**, **Imperative**, **Functional** (with ADTs and immutable structures)                                     |
| **Key Differentiator**  | **Code-as-Object (The Meta Core)**: The Abstract Syntax Tree (AST) is directly exposed as native, manipulable objects at runtime. |
| **Typing**              | **Dynamic by default**, with **optional static typing** and gradual type inference                                                |
| **Performance**         | **Bytecode VM** with **JIT compilation** for hot paths, built on Rust for safety                                                  |

## Syntax Overview

METOREX syntax prioritizes readability while minimizing keystrokes, combining elements from Ruby.

### Basic Syntax

See [examples/basic_syntax.rb](examples/oop/basic_syntax.rb)

### Exception Handling

See [examples/exception_handling.rb](examples/advanced/exception_handling.rb)

### Pattern Matching

See [examples/pattern_matching.rb](examples/advanced/pattern_matching.rb)

### Traits (Interfaces)

See [examples/traits.rb](examples/advanced/traits.rb)

### Optional Type Annotations

See [examples/type_annotations.rb](examples/advanced/type_annotations.rb)

## Meta-Programming: The Core Innovation

METOREX exposes the program's structure as native objects, eliminating the need for external `eval` functions.

### Code-as-Object Hierarchy

The parser converts source code into an in-memory graph of objects, defined in the Rust core and exposed in Metorex.

| Metorex Class Name   | Role                                                                                                              | Example of Manipulation                    |
| :------------------- | :---------------------------------------------------------------------------------------------------------------- | :----------------------------------------- |
| **`BlockStatement`** | **The Core Meta-Object.** Represents a sequence of code lines (a method body, loop body, or implicit code block). | `block.call` to execute the code.          |
| **`Assignment`**     | Represents `x = 10`.                                                                                              | `.target` to see the variable name.        |
| **`MethodCall`**     | Represents a function/method invocation.                                                                          | `.receiver` and `.args` for code analysis. |

### Implicit Block Capture and Execution

Methods can accept code blocks as objects.

See [examples/metaprogramming/blocks_as_objects.rb](examples/metaprogramming/blocks_as_objects.rb)

### Dynamic Method Definition

See [examples/dynamic_method_definition.rb](examples/advanced/dynamic_method_definition.rb)

### Building DSLs

See [examples/dsl_example.rb](examples/advanced/dsl_example.rb)

## Architecture

### Multi-Phase Execution Model

1. **Phase 1 (MVP)**: Direct AST interpretation for rapid development
   - Lexer → Parser → AST → Interpreter
   - Full meta-programming capabilities
   - Exception handling, pattern matching, testing

### Runtime Components (MVP)

- `VirtualMachine` (`src/vm.rs`) seeds the AST interpreter with the environment stack, global object registry, call stack, heap placeholder, and built-in class initialization.

2. **Phase 2**: Bytecode compilation for performance
   - AST → Bytecode Compiler → VM
   - Reflection and runtime definition
   - Traits and advanced OOP

3. **Phase 3**: Production optimizations
   - JIT compilation for hot paths (LLVM)
   - Full concurrency support (threads, channels, atomics)
   - Optional type system with inference
   - Comprehensive standard library

4. **Phase 4**: Advanced features
   - Macro system for compile-time metaprogramming
   - Algebraic data types and functional features
   - WebAssembly compilation target
   - Security features and sandboxing
   
   ## Design Principles

| Principle             | Implementation                                                                         |
| :-------------------- | :------------------------------------------------------------------------------------- |
| **Syntax Simplicity** | Non-whitespace sensitive with mandatory `end` blocks. No colons, optional parentheses. |
| **OO Purity**         | Everything is an object rooted in `Object` class. No standalone functions.             |
| **Meta-First**        | AST is always accessible as first-class objects. Code can inspect and modify itself.   |
| **Gradual Typing**    | Dynamic by default, optional static types for performance. Best of both worlds.        |
| **Performance**       | Rust-based VM with bytecode compilation and JIT for hot paths.                         |
| **Safety**            | Exception handling, memory safety from Rust, optional sandboxing.                      |
| **Concurrency**       | Multiple models: fibers, async/await, OS threads, channels. Choose the right tool.     |
| **Productivity**      | Built-in testing, documentation, linting, formatting. Everything you need included.    |

## Standard Library Highlights

### Networking

See [examples/networking.rb](examples/advanced/networking.rb)

### Concurrency

See [examples/concurrency.rb](examples/advanced/concurrency.rb)

### Serialization

See [examples/serialization.rb](examples/advanced/serialization.rb)

## Roadmap Highlights

See [ROADMAP.md](ROADMAP.md) for complete details.

### Phase 1: MVP (In Progress)
- Lexer and Parser
- AST Interpreter
- Expression evaluation (arithmetic, collections, indexing)
- Logical operators (`&&`, `||`) with short-circuit evaluation
- Logical NOT operator (`!`)
- Scope resolution (`::`) for class constants
- Global variables (`$variable`)
- Method dispatch for built-in objects
- Meta-programming core
- Exception handling
- Pattern matching (`case/when` and `case/in` with Ruby 2.7+ `=> name` binding)
- Keyword arguments (`def method(name:, age: 10)` and `method(name: "Bob")`)
- Operator method names (`def +(other)`, `def ==(other)`, `def [](key)`, `def []=(key, value)`)
- Module and mixin support (`module`, `include`, `extend`)
- `define_method` for dynamic method definition on classes, taking a block, a Proc, a `Method`, or an `UnboundMethod`. It returns the method name as a Symbol, inherits the current `private`/`public` visibility when called from inside the target module, always makes `initialize` private, fires the `method_added` hook, and raises `FrozenError` on a frozen module
- `define_singleton_method` for defining a method on a single object's singleton class, accepting the same block, Proc, `Method`, or `UnboundMethod` bodies as `define_method`
- Bodies installed by `define_method` follow lambda control flow: `return`, `break`, and `next` finish the method with a value, and `redo` re-runs it
- Proc and Method objects: `Kernel#proc`, `Kernel#lambda`, `Proc.new`, `Proc#lambda?`, `Symbol#to_proc`, `Method#to_proc` (which stays bound to its original receiver), `Method#unbind`, and `Method#owner` (returns the defining module)
- `Object#method` converts its name argument with `#to_str`, and builds a `method_missing` dispatcher for a name the object claims through `respond_to_missing?`
- `Object#public_method` does the same lookup but raises `NameError` for a private or protected name, and asks `respond_to_missing?` without the private flag
- Procs and lambdas are distinct kinds. A lambda checks its arity and its `return` returns from the lambda, while a proc pads missing arguments with nil, drops extras, and its `return` returns from the method that created it
- `Kernel#loop` runs its block until `break` (whose value the loop returns) or until the block raises `StopIteration` or one of its subclasses. Every other exception propagates
- `method_missing` hook for intercepting undefined method calls
- Runtime class modification: `remove_method`, `undef_method`, `alias_method`, `module_function`
- Constant visibility on a module receiver: `private_constant`, `public_constant`, and `deprecate_constant` (which returns the receiver and raises `NameError` for an undefined name). Reading a deprecated constant through `::`, `const_get`, or `remove_const` warns once the `Warning[:deprecated]` category is switched on
- `Warning[:category]` and `Warning[:category] = bool` for reading and setting the warning category switches. Like MRI, `:deprecated` starts off
- Class variables: `class_variable_set`, `class_variable_get`, `class_variable_defined?`, `class_variables` (lookup walks included modules and superclasses; `class_variables(false)` lists only own names)
- Module ancestry comparison with `<=>`: `-1` when the receiver is a descendant or includer of the argument, `+1` when it is an ancestor or included-by, `0` when they are the same module, and `nil` when unrelated or the argument is not a module
- Reflection: `class`, `instance_of?`, `is_a?`, `itself`, `respond_to?`, `send`, `instance_variables` (Symbols, in declaration order), `instance_variable_get`, `instance_variable_set`, `local_variables`
- `Object#methods` reports `def obj.name`, `class << obj`, `define_singleton_method`, and the modules `extend` attached, leaving out private ones and anything `undef_method` removed
- `Symbol` is its own class rather than an alias of `String`, keeping String's character-level methods (`length`, `upcase`, `start_with?`)
- `Array#&` and `Array#|` for intersection and union, both dropping duplicates
- `!~` dispatches `=~` on the receiver and negates the result, raising `NoMethodError` when the receiver has no `=~`
- `object_id` identifies a reference type by its address and an immediate by its value, so two equal Symbol, String, Integer, or Float literals share an id
- Hashes iterate in insertion order. A reassigned key keeps its position and a deleted one leaves the rest in place
- `Hash#each_pair`, Ruby's alias for `Hash#each`
- `Object#public_methods`, `Object#private_methods`, and `Object#protected_methods` report the methods of that visibility, including those a `class << obj` or `extend` supplied, and, unless passed false or nil, the ancestors' and mixins'. On a class they walk the class-method chain
- `Integer#divmod` returns the floored quotient and the modulus, with the signs following the divisor
- `Kernel#rand` draws a Float in [0, 1) with no argument, an Integer below a given bound (whose sign it ignores), or a value from a Range, answering nil for a backwards one. It converts other arguments with `#to_int`
- `Kernel#srand` installs a seed and answers the one it replaced, picking a seed of its own when given none. It converts its argument with `#to_int`, and the same seed repeats a whole sequence
- `Kernel#readline` reads a line and raises `EOFError` at end of input, where `gets` answers nil. `Kernel#readlines` collects every remaining line into an Array
- `Kernel#remove_instance_variable` takes a variable off an object and answers what it held, raising `NameError` for one that is not defined and `FrozenError` on a frozen receiver
- `Kernel.instance_methods` lists Kernel's native methods, with the private ones reported as private
- `respond_to?` falls back to `respond_to_missing?` for a name the lookup missed, passing along the private flag it was given. Every object carries a default `respond_to_missing?` that answers false
- `Method#owner` answers the module itself for a native Kernel method, not its name
- A class reports and enforces visibility on its natively-implemented methods, so `private_class_method :new` makes `respond_to?(:new)` false and `Klass.new` raise `NoMethodError`
- `singleton_class` answers `NilClass`, `TrueClass`, or `FalseClass` for those three objects, raises `TypeError` for an Integer, Float, or Symbol, and is frozen when the object is
- `-"str"` and `+"str"` both answer the string, matching Ruby's deduplicated and mutable forms
- `singleton_method` looks only at the singleton layer: a `def obj.name`, a class method, and the modules `include`, `prepend`, or `extend` attached. A method the object's class defines raises `NameError`
- `singleton_methods(all = true)` reports the same layer as names, adding what the ancestors' singleton classes supply. Passing false leaves out both the ancestors and the modules `extend` attached
- `extend self` inside a module body, and `extend Mod` at the top level
- `ary[range]` slices an Array, counting a negative bound from the end and answering nil for a start past the end
- `sprintf` and `format` convert their format argument with `#to_str` and raise `TypeError` otherwise, and `%s` renders a Symbol without its colon
- `Float::INFINITY`, `NAN`, `EPSILON`, `MAX`, `MIN`, `DIG`, and `MANT_DIG`
- `tap`, `then`, and `yield_self` raise `LocalJumpError` when given no block, and `throw` raises `ArgumentError` for the wrong argument count
- `Numeric` is a real superclass of `Integer` and `Float`
- An Integer and a Float compare against each other, so `(0...1).include?(0.38)` and `0.38 <=> 0` answer correctly
- `class << target = value` assigns first, then opens the singleton class of what was assigned
- `%i[a b c]` and `%i(a b c)` build an Array of Symbols, alongside `%w` for Strings
- `puts [1, 2].inspect` passes the array with its trailing method call, the way Ruby reads a paren-less argument that starts with `[`, while `values[0]` on a bound name still indexes
- An `if`, `unless`, or assignment as the last statement of a block or branch is that block's value
- `Kernel#raise` is a method as well as a keyword, so `send(:raise, ...)`, `Kernel.raise`, `method(:raise)`, and a singleton that makes it public all reach it. A bare `raise` with nothing to re-raise gives `RuntimeError: unhandled exception`
- `=~` and `!~` match a Symbol against a Regexp on the characters it is named with
- `Kernel#proc` is reachable through `send` and hands back an existing Proc unchanged, keeping a lambda a lambda. Without a literal block it raises `ArgumentError`
- `equal?` compares reference types by address, so two Procs, Sets, or Exceptions are only equal when they are the same object
- An array literal gathers trailing `key: value` or `key => value` pairs into one Hash as its last element, so `[1, a: 2, b: 3]` is `[1, {a: 2, b: 3}]`
- `Kernel#p` writes each argument's `inspect` on its own line, honoring a user-defined `inspect`, and returns the argument, the argument list, or nil for none
- `puts`, `print`, `p`, and `warn` write through `$stdout` and `$stderr`, so assigning an object with its own `write` captures the output. `print` with no arguments writes `$_`
- `puts` and `print` render with `to_s`, so a Symbol prints without its colon, while `p` and `inspect` keep it
- A bare method call reaches the method on `self` before a same-named Kernel function, so a class defining `to_s` can call it bare from another of its methods
- `Kernel#trace_var` runs a hook every time the named global is assigned, taking the hook as a block, a Proc, or a String of code to evaluate, and raising `ArgumentError` when given none. `Kernel#untrace_var` drops every hook on a global, or just the one it is handed. A `:$name` symbol names the global for both
- `Kernel#warn` writes through `Warning.warn`, which is defined in Ruby so a program can replace it, and stays silent while `$VERBOSE` is nil. Each argument warns on its own line, as does each element of an Array argument, and a message already ending in a newline keeps just the one. `uplevel:` prefixes `path:line: warning: ` taken from that many frames out, `category:` converts through `to_sym`, and a negative uplevel raises `ArgumentError`
- `**hash` in a call passes keyword arguments rather than a positional Hash, so an empty Hash contributes no argument at all
- A required parameter written after an optional one binds from the end of the argument list: `def pad(prefix = "<", value)` called with one argument fills `value`
- `Method#source_location` answers `[path, lineno]`, and a bare `method(:name)` inside an instance method resolves against `self`
- `Enumerator` steps through the values a method yields: `to_enum`/`enum_for` build one over any method that yields, `next` and `peek` walk it, `rewind` restarts it, and running past the end raises `StopIteration`. A method that yields answers one when called without a block, which is what `then` and `yield_self` return
- A method body and a class body each own their locals. An assignment inside one defines a new local there rather than reaching a same-named variable outside it, and a block still sees and assigns the locals of the scope it was written in
- `send`, `__send__`, and `public_send` raise `ArgumentError` with no method name and `TypeError` for a name that is neither a Symbol nor a String. `BasicObject` reports its own instance methods, `__id__` and `__send__` among them
- `super` from a method defined in an anonymous module reaches the next module in the chain, because each running method records the module it was defined in rather than being placed by name
- A splat inside an array literal splices its elements in place, so `[first, *rest]` flattens the one level Ruby flattens
- A class descending from `BasicObject` does not reach top-level constants, since Ruby finds those among Object's own. A leading `::` reads the top level directly, in a reference, in an `include`, and on the left of an assignment
- `Object.constants` lists every top-level constant, `BasicObject` holds the constant naming itself, and `instance_of?` reports a Class as an instance of `Class` and a Module of `Module`
- Integer bit operations: `<<` and `>>` shift (a negative count shifts the other way, and a count past the word width leaves 0 or the sign bit), `~` complements, and `bit_length` counts the bits a value needs. `:>>` and `:~` are symbol names like any other operator
- The default `initialize` takes no arguments, so `new` given any raises `ArgumentError`. Every arity error reads the way Ruby's does: `wrong number of arguments (given 0, expected 2)`, `(given 0, expected 1..2)` when a parameter has a default, and `(given 0, expected 1+)` when one is variadic
- Integers are arbitrary precision. A literal, a sum, a product, or a power past the machine word keeps its exact value rather than saturating or turning into a Float, and a result that fits again narrows back, so `(2 ** 64) - (2 ** 64)` is the same `0` any other expression produces. Comparison, sorting, `divmod`, the shifts, `abs`, `bit_length`, `Float#to_i`, `Kernel#Integer`, and `Rational` are all exact at any size. Two separately built values of the same large size are separate objects, as Ruby's are
- Dividing by zero raises `ZeroDivisionError` reading `divided by 0`
- `expr rescue fallback` answers the fallback when the expression raises a `StandardError`. On the right of an assignment the modifier binds to the value, so `value = risky rescue nil` assigns nil
- `instance_eval` takes source as well as a block: either runs with `self` bound to the receiver. The block form yields the receiver and takes no other arguments, the source form takes one to three
- `raise(SomeError, "message")` parses with parentheses, not only in its paren-less form
- `instance_exec` raises `LocalJumpError` without a block, reports an arity of -1, and refuses a `def` in a block run against an immediate, where a singleton method cannot exist
- A call that visibility refuses reaches a user-defined `method_missing` before raising, the way Ruby's does, and `super` from an override of it raises the NoMethodError the default would
- `NoMethodError` and `NameError` carry `#name` and `#receiver`, so a rescue can tell which method was called and on what
- `!=` is the negation of `==`, so a class that defines only `==` gets both, and `send(:!=, other)` reaches the same definition
- `singleton_method_added` fires for every way a singleton method arrives: `def obj.name`, a `class << obj` body, `alias`, `alias_method`, `define_method`, and `define_singleton_method`. Undefining the hook makes the next definition raise `NoMethodError`, or reach `method_missing` when one is defined
- A `class << obj` body answers with its own last value rather than re-running the last statement, so a side effect there happens once
- A `def` nested inside a block or a `begin` within a class body installs on that class, where it used to be an internal error
- `singleton_method_removed` fires when a singleton method goes, and `remove_method` in a `class << Klass` body reaches a method that `def Klass.name` put there
- `singleton_method_undefined` fires the same way for `undef_method`, which retires the method so `respond_to?` answers false and a call raises `NoMethodError`
- `Exception#to_s` is the message alone, and the class name when there is no message. A message argument that is not a String is rendered with `to_s`, which any object may define
- A bare `raise` followed by a trailing comment is still the re-raise form
- `Exception#backtrace` is nil until the exception is raised, then answers the same Array every time, so an update through it sticks. `Exception#set_backtrace` takes nil, a String, or an Array of Strings, keeping the very Array it is handed, and refuses anything else with a `TypeError`
- A rescue clause sets `$!` to the exception it is handling and `$@` to that exception's backtrace
- `Exception#backtrace_locations` answers `Thread::Backtrace::Location` objects carrying a `path` and a `lineno`, nil until the exception is raised, and the same Array on every call. `set_backtrace` accepts those objects as well as Strings
- `Array#each_with_index` yields each element with its position, and answers an Enumerator without a block
- The `Errno` namespace is populated: every `Errno::EXXX` is a subclass of `SystemCallError` carrying the platform's own number in its `Errno` constant, taken from libc rather than a table. `SystemCallError.new(message, errno)` answers the class that number names, and `#errno` reads it back
- `===` on an object reaches a user-defined `===` or `method_missing` before the default, and `:===` is a symbol name like any other operator
- `Exception#cause` is the exception a rescue clause was handling when this one was raised, set once and never to the exception itself. An error the interpreter raises inside a rescue body records what it followed just as an explicit `raise` does
- `is_a?` and `kind_of?` walk an exception's real ancestry, so a rescued `ZeroDivisionError` answers true for `StandardError`
- `Exception#detailed_message` decorates the message with the class name, stands in with `unhandled exception` or the class name for an empty one, and takes a `highlight:` keyword. `Exception#full_message` renders the backtrace with it, `order:` deciding which end the message sits at, and honors a class that defines its own `detailed_message`
- An exception carries the class it was built from, so an anonymous `Class.new(RuntimeError)` subclass reports it through `#class`, `is_a?`, and `===`
- `def exception.name` installs a singleton method, and each exception gets its own singleton class
- An exception subclass runs its own `initialize` and carries the instance variables it sets, so `attr_reader` on one works. `dup` copies that state along with the message, backtrace, and cause, calls `initialize_copy`, and leaves the singleton class behind
- Two exceptions are equal when they share a class, a message, and a backtrace, so a copy equals its original
- Each `Errno::EXXX` carries the message its number stands for, so `Errno::EINVAL.new.message` reads `Invalid argument`. A custom message and location are appended as `<default> - <custom>` and `<default> @ <location> - <custom>`, and a subclass inherits the default
- `Exception#exception` answers self with no argument or when handed self, and otherwise a copy carrying the new message, without re-running `initialize`. `Exception.exception` is another name for `new`
- An exception built with no message reports its class name, where one built with an empty message reports that. A bare `raise` makes the second kind, and the "unhandled exception" wording belongs to the uncaught report
- Modifying a frozen object raises `FrozenError` naming the class and inspecting the object, and the error carries it as `#receiver`, which `FrozenError.new` also takes by keyword. When inspecting would itself modify the object, the message shows `...`
- An Array, Hash, or Set can be frozen, and every method that changes one in place refuses a frozen receiver
- A backtrace entry reads `file:line:in 'label'`, the way Ruby's does. `Exception#full_message` appends the cause chain after the exception's own report
- A block parameter list takes `*`, `**`, and `&` without naming them, so `{ |**| }` accepts and discards keywords
- `String#lines` splits on the line separator, keeping it on each piece
- An Array index past either end answers nil rather than raising, and a negative one counts back from the end
- The built-in exception hierarchy is complete, rooted at Object the way every class is. `NoMemoryError`, `SecurityError`, `SystemStackError`, `FiberError`, `ThreadError`, and `ClosedQueueError` are defined, and `Interrupt` sits under `SignalException`
- `Exception#inspect` reads `#<ClassName: message>`, using whatever `to_s` answers, and the class name alone when that is empty
- `Signal.list` and `Signal.trap`, with `Process.kill` running the handler in force when the target is this process. The default disposition raises `Interrupt` for SIGINT and `SignalException` elsewhere, both answering `signo` and `signm`
- `Signal.signame` answers the name a number goes by, nil for a number no signal uses, and asks anything but an Integer for `to_int`. `Signal.list` carries `CLD` alongside `CHLD` at the same number, and a lookup by number answers the name the signal goes by rather than the older spelling
- `Warning.categories` lists the categories `Warning[]` accepts, which are `:deprecated`, `:experimental`, `:performance`, and `:strict_unused_block`. `:experimental` starts on and the rest start off, `-w` turns `:deprecated` on, a category Ruby does not know raises ArgumentError, and anything but a Symbol raises TypeError
- `Warning` extends itself, so `warn` is an instance method it answers to. A program that writes its own `def Warning.warn` overrides it and reaches the built-in through `super`, and every interpreter warning travels through whichever one is in force. A warning in a category that is switched off never reaches it at all
- A key written twice in one Hash literal is reported once for the literal as written, naming the line whose value wins, so a literal inside a loop is reported the once
- `IO::WaitReadable` and `IO::WaitWritable`, with the four `IO::EAGAINWaitReadable`-style classes that pair them with `Errno::EAGAIN`. The `EWOULDBLOCK` spellings name the same classes, since the two errno values match
- `KeyError.new` takes `receiver:` and `key:`, read back through `#receiver` and `#key`. A KeyError with no key recorded raises ArgumentError from `#key`, the way Ruby does
- `LoadError#path` answers the feature that could not be loaded, and nil on a LoadError raised by nothing in particular. A `require_relative` of a missing file raises LoadError rather than RuntimeError
- `SyntaxError#path` answers the file the unparsable code came from: the filename handed to `eval`, or the file a `require` could not parse. It is nil on a SyntaxError constructed directly. Assigning to something that is not a variable, a constant, an element, or an attribute, such as `1 + 1 = 2`, is refused while parsing rather than at run time
- `SystemCallError.new` reads its number from the second argument, or from the first when that is the only one, and answers an instance of the `Errno` class the number names. The message is what the number stands for, with a custom message and a location folded in as `default @ location - custom`. A number no class names stays a SystemCallError, and `#errno` answers it either way. A Float or a real Complex truncates to a number, a Complex with an imaginary part raises RangeError, and a message or number of the wrong type raises TypeError
- `Errno` covers every errno the platform names, each class carrying its number in an `Errno` constant and reporting the message the C library gives for it
- `SystemExit.new` takes the status first, where true stands for a clean exit and false for a failed one, and the message second. An exception built without a message reports its class name, and `#status` and `#success?` answer for a subclass of SystemExit as well as for SystemExit itself
- Raising a subclass of SystemExit ends the program with the status it carries and reports nothing, the way raising SystemExit does
- `UncaughtThrowError#tag` and `#value` report the tag a `throw` was called with and the value it carried when no live `catch` held that tag, and are nil on one raised by no throw at all
- A Symbol names any operator method, `:/`, `:**`, `:!`, `:&`, `:|`, and the unary `:-@` and `:+@` among them, so `send` reaches one by name. A slash glued to the colon names the division method, while a colon with a space before the slash still opens a regular expression, as in `condition ? value : /pattern/`
- `Integer#-@`, `Integer#+@`, `Float#-@`, and `Float#+@` answer to the names `send` reaches the unary operators by
- `Integer#allbits?`, `#anybits?`, and `#nobits?` report how the bits of a mask sit in the receiver, comparing as arbitrary-width integers so a bignum and a negative number answer the way a fixnum does. The mask is asked for `to_int`, and anything that cannot become an Integer raises TypeError
- `Integer#ceil`, `#floor`, `#truncate`, and `#round` take a precision. Zero or more leaves the number alone, and a negative one rounds to that power of ten. `round` also takes `half:`, which sends a value sitting exactly between two multiples away from zero (`:up`, the default), toward it (`:down`), or to the even multiple (`:even`)
- `Integer#gcd`, `#lcm`, and `#gcdlcm` answer positive numbers whatever the signs they were handed, and refuse a non-Integer argument
- `Integer#ord`, `#to_int`, and `#magnitude` join `#to_i` and `#abs`, and `String#ord` answers the codepoint of the first character
- An operand a number does not know is handed to that operand's `coerce`, and the operator is applied to the pair it answers. This covers the arithmetic, bitwise, and ordering operators. An error raised inside `coerce` travels out to the caller rather than becoming a TypeError
- A bignum is ordered against a Float exactly, rather than by rounding the bignum to a Float first, so `(2 ** 64 + 38) <= (2 ** 64 + 38.0)` is false
- A sign glued to a numeric literal belongs to the literal, so `-1.abs` is `(-1).abs`. An exponent still binds tighter, leaving `-2 ** 2` as `-4`. In a paren-less call the same spacing rule applies: `foo -1` passes a negative number where `foo - 1` subtracts one
- Dividing two integers answers an integer, rounded toward negative infinity, and `%` leaves a modulus carrying the sign of the divisor, so `-5 / 2` is -3 and `-5 % 3` is 1. An Integer refuses a zero divisor whether or not the divisor is a Float, where a Float receiver answers an infinity or NaN
- `Integer#div`, `#modulo`, `#divmod`, `#remainder`, `#fdiv`, and `#ceildiv` divide and report the part each names. `remainder` truncates where `modulo` floors, so it carries the sign of the receiver. `fdiv` takes the quotient with the extra bits the two widths carry, so a pair of bignums that would each round to infinity still answers the ratio between them
- `Integer#coerce`, `#digits`, `#numerator`, `#denominator`, `#to_r`, `#rationalize`, and `#size`, with `Integer.sqrt` and `Integer.try_convert` on the class
- A number compared against something that is not one answers `==` and `===` by asking that object instead, and has no `<=>` ordering against it, which reads as nil. `Comparable` is mixed into Numeric, Integer, Float, and String
- `Foo::bar` names a method when the name is lowercase, the way `Foo.bar` does, and a constant when it is capitalized
- `Float#truncate` drops the fraction, and a non-finite Float reads as `Infinity`, `-Infinity`, or `NaN` through `to_s` as well as in an interpolation
- `Rational#floor` and `#ceil` round an exact fraction the two ways
- `Float#zero?`, `#positive?`, `#negative?`, `#magnitude`, `#to_int`, `#coerce`, `#numerator`, `#denominator`, `#quo`, `#fdiv`, `#divmod`, `#modulo`, `#angle` (with `#arg` and `#phase`), `#next_float`, and `#prev_float`
- `Float#ceil`, `#floor`, and `#truncate` take a precision: a positive one keeps that many digits after the point and answers a Float, and zero or less answers the whole number those digits sit in. A number with no whole part to take, NaN or an infinity, raises FloatDomainError
- Nothing compares against NaN, which `<`, `>`, `<=`, and `>=` report as false rather than as a failed comparison, whatever the width of the other number
- `eql?` is equality without conversion, so `1.0.eql?(1)` is false where `1.0 == 1` is true
- `%` refuses a zero divisor whatever the receiver, where `/` answers an infinity
- A Float is a binary fraction, and that exact value is the Rational it stands for, so `Rational(0.3)` is not (3/10). `Math::PI` and `Math::E` are the constants Math carries
- `Numeric` carries the protocol every number answers, written in terms of the methods a subclass supplies: `abs` and `magnitude`, `ceil`, `floor`, `round`, `truncate`, `to_int`, `zero?`, `nonzero?`, `positive?`, `negative?`, `integer?`, `finite?`, `infinite?`, `real?`, `real`, `imaginary` (with `imag`), `conjugate` (with `conj`), `div`, `modulo` (with `%`), `divmod`, `remainder`, `fdiv`, `eql?`, and `coerce`. Integer and Float reach their own implementations first, so what Numeric defines serves the subclasses a program writes
- `def -@` and `def +@` name the unary operators, and a sign in front of an object calls the one it defines. A Rational answers them too
- `Math` carries the usual functions: `sqrt`, `cbrt`, the trigonometric and hyperbolic families with their inverses, `exp`, `log` with an optional base, `log2`, `log10`, `log1p`, `expm1`, `hypot`, `atan2`, `frexp`, `ldexp`, `erf`, and `erfc`. Each is a module function, so `Math.sqrt` and a private `sqrt` inside a class that includes Math reach the same one. An argument outside a function's domain raises `Math::DomainError`, and one that is not a number raises TypeError
- A logarithm of an integer too wide for a Float keeps the digits the Float cannot hold, so `Math.log2(2 ** 10001)` is 10001.0 rather than an infinity
- Two Floats are equal when they are the same number, which is what Ruby compares: `0.1 + 0.2 == 0.3` is false, and a number as small as 1e-16 is not zero
- The built-in modules report themselves as modules, so `module_function` works inside one and `Module#===` recognizes it
- `Rational#ceil`, `#floor`, `#truncate`, and `#round` take a precision, which moves the decimal point: a positive one keeps that many places and answers a Rational, and zero or less answers the Integer those places sit in. `round` also takes `half:`. The precision must be an Integer, and Ruby refuses an object carrying `to_int` rather than converting it
- A Rational hands an operand it does not know to that operand's `coerce`, the way the other numbers do, and `Rational#to_f` divides with the precision the two widths carry
- Complex arithmetic: `+`, `-`, `*`, `/`, `quo`, `fdiv`, and `**` with a whole exponent, composed from the two parts of each operand, so exact parts stay exact and dividing two Integers answers the Rational between them. A real number on either side is that number with nothing on the imaginary axis, and an operand that is neither is asked to `coerce`
- `Complex#abs` (with `#magnitude`), `#abs2`, `#arg` (with `#angle` and `#phase`), `#polar`, `#rect` (with `#rectangular`), `#conjugate` (with `#conj`), `#-@`, `#eql?`, `#coerce`, `#finite?`, `#infinite?`, and the conversions `#to_f`, `#to_i`, `#to_r` and `#rationalize`, which refuse a number with anything left on the imaginary axis. A Float part is never exact enough for `to_f` and `to_i` to drop, so `Complex(1, 0.0).to_i` raises where `Complex(1, 0).to_i` answers 1
- `Complex#to_s` and `#inspect` render each part with the method they were asked for, and put a `*` before the `i` when the imaginary half does not end in a digit
- `Complex::I` is the imaginary unit. `Complex#numerator` scales both parts to the denominator they share, `#denominator` answers that denominator, and `#<=>` orders two Complexes that have nothing on the imaginary axis while answering nil for anything else. `#eql?` compares the parts by class as well as by value, and `#positive?` and `#negative?` are undefined, since a Complex names no point on the number line
- `Numeric#dup` and `#clone` answer the number itself, and a clone that asks for `freeze: false` raises ArgumentError. `#numerator` and `#denominator` read the fraction the number stands for, `#+@` answers self, and defining a singleton method on a number raises TypeError
- A number is not built by hand: `Float.new`, `Rational.new`, `Complex.new` and `Integer.new` do not exist, and `allocate` has nothing to allocate
- A compound assignment is an expression, so `(count += 1)` answers what it assigned
- Comparing two collections remembers the pairs already in flight, so an array that holds itself compares without following the cycle forever
- `Array#rindex`, `#concat`, `#delete_at`, `#delete_if`, `#keep_if`, `#each_index`, `#reverse_each`, `#values_at`, `#intersect?`, `#assoc`, `#rassoc`, `#to_ary` and `#deconstruct`, with `#first` and `#last` taking a count
- Array's in-place methods: `#compact!`, `#reverse!`, `#sort!`, `#sort_by!`, `#map!`/`#collect!`, `#reject!`, `#select!`/`#filter!`, `#uniq!`, `#rotate!`, `#shuffle!` and `#flatten!`, each answering nil when nothing changed where Ruby does
- `Array#at`, `#count`, `#take_while`, `#drop_while`, `#rotate`, `#shuffle`, `#sample`, `#to_a`, `#entries`, and `#uniq` with a block that names the key
- `Array#min`, `#max`, and `#minmax` order with `<=>` or a block, and raise ArgumentError when two elements cannot be compared
- `Array#fetch`, `#fetch_values`, `#insert`, `#union`, `#intersection`, and `#difference`, with `#pop` and `#shift` taking a count
- `Array#<=>` orders element by element, answering the first non-zero result verbatim, and two arrays that reach themselves compare rather than recursing
- `-`, `&`, `|`, `union`, `intersection`, and `difference` match elements with `eql?` the way Ruby does, and the same object counts whatever its `eql?` says
- `+`, `-`, `&`, `|`, `zip`, `transpose`, `assoc`, and `rassoc` put an operand that is not an Array through `to_ary`, while an Array subclass is taken as the array it already is
- `[1, 2] * 3` repeats the array and `[1, 2] * ", "` joins it
- `Array#values_at` takes Ranges, including endless and beginless ones, and answers nil for a position the array has no element at
- `Array#transpose` raises IndexError when the rows are not all the same length, and `#first`/`#last` raise RangeError for a count too large for a machine word
- `Array#flatten` descends all the way down, or as many levels as its argument names, and raises ArgumentError on an array that contains itself
- A subclass of Array holds real elements: `Sub.new` runs the subclass's own `initialize` against storage that is already in place, `Sub[1, 2]` fills it without calling `initialize`, and an instance answers Array's methods and compares equal to a plain Array with the same contents
- `each`, `map`, `select`, `filter`, `reject`, `map!`, `select!`, `reject!`, `sort_by!`, `take_while`, and `drop_while` answer an Enumerator without a block, and a walk may append to the array it is walking
- An array or hash that contains itself inspects as `[...]` or `{...}` rather than recursing, and comparing two of them answers rather than running forever
- `Array#eql?` compares element by element with `eql?` semantics, so `[1]` and `[1.0]` differ
- The search methods ask each element whether it is `==` to what they were given, so an object that defines `==` decides for itself, and `#delete` runs its block when nothing matched
- A method called without the block it would have yielded to answers an Enumerator over the same walk, reporting the collection's size ahead of time where the walk covers all of it
- `#delete_if` and `#keep_if` move the survivors forward and cut the array at the end, so it keeps its length while the block runs and holds what was already decided if the block raises
- `Array#sort` takes a comparison block, and `Enumerable` is mixed into the classes whose values can be walked
- An operator at the end of a line carries the expression onto the next one, `alias` names an operator method, and `Array#concat` and friends take anything that answers `to_ary`
- An exception raised while a file loads keeps its class and message on the way out of `require`, `require_relative`, and `load`, so a `rescue ArgumentError` around the load catches it
- `main.using` reaches the top-level `using` through an explicit receiver, and raises RuntimeError when the call sits inside a class or module body
- `Exception#message` dispatches `to_s`, so a subclass or a singleton that redefines `to_s` decides its message. `raise SomeClass` instantiates the class, which keeps a subclass `initialize` and `to_s` in force
- `NameError#name` is set on every path that raises one: an undefined variable or method, a constant, and an unset class variable. `instance_variable_get` and `class_variable_get` report back the very name object they were handed
- Class variables are looked up through the superclass chain, and reading one that was never assigned raises NameError instead of answering nil
- `NameError.new` takes a name as its second argument and a `receiver:` keyword, and `#dup` carries both to the copy. An undefined name records the object the lookup was made on
- `NoMethodError#args` answers the arguments the failed call was made with, and `NoMethodError.new` takes them as a third argument
- `String#*` repeats a string, raising ArgumentError on a negative count
- `NameError#receiver` is set on every path that raises one: a method call, a bare or namespaced constant, an unset class variable, and `instance_variable_get` / `class_variable_get`. Asking an exception that has none raises ArgumentError, as Ruby does
- `StopIteration#result` answers what the underlying `each` returned once an Enumerator runs out
- `Thread::Backtrace::Location` answers `label`, `base_label`, and `absolute_path` alongside `path` and `lineno`, and renders as `path:lineno:in 'label'`
- `SignalException.new` is named by the signal it stands for, taking a number, a name, or a symbol with or without the `SIG` prefix, with a second argument replacing the name. An argument that names no signal raises ArgumentError
- A bare `rescue` catches StandardError rather than everything, so an Exception outside that family goes past it
- `StringIO` collects what is written to it and hands back the string, and reads through it a line or a length at a time
- `printf` writes a formatted string to `$stdout`, or to an IO given as its first argument
- `require` of a library metorex provides itself, such as `stringio`, answers without looking for a file
- `File.new` opens a file the way `File.open` does when given no block
- `pp` prints the same inspect form `p` does and answers its argument the same way
- A Hash renders the way Ruby shows one: a Symbol key as `name: value` and every other kind as `key => value`, with the bookkeeping entries left out
- Reassigning `$stdout` to a File handle sends `p`, `puts`, and `print` there
- `Array#inspect` renders each element through its own `inspect`, so an object that defines one is shown the way it asks to be
- `File#read` takes a length and leaves the rest for the next read
- `Kernel#open` opens a file by path, taking one from anything answering `to_path` or `to_str`, and hands it to a block when given one. An argument answering `to_open` is asked to open itself and its answer is what comes back
- `File#gets` and `#readline` read a line at a time, and `#read` picks up where they left off
- The `File::CREAT` family of open flags
- `Enumerator.new { |yielder| ... }` builds an enumerator from a generator block, with `yielder <<` and `yielder.yield` collecting what it produces
- `loop` without a block answers an enumerator that yields forever and reports `Float::INFINITY` for its size. With a block, a StopIteration ends it and the loop answers the result the finished iterator carried
- `load` runs the file every time and leaves `$LOADED_FEATURES` alone, while `require` answers false only for a file that list still names. Both take a path from anything answering `to_path` or `to_str`, expand a leading `~` against `ENV["HOME"]`, and name an absolute or `./`-relative path outright rather than searching `$LOAD_PATH`. A file that does not exist or cannot be read raises LoadError
- `load(path, true)` runs the file inside a fresh anonymous module and `load(path, SomeModule)` inside that one, so its constants and top-level methods land there rather than on Object
- `$LOAD_PATH` entries may be objects answering `to_path`
- `File::Separator` and its siblings, `File.chmod`, and `Process.euid` / `Process.uid`
- An endless definition, `def name = expression`, defines a method whose body is that expression, with or without parameters and for a class method as readily as an instance one
- A writer may name its parameter without parentheses, as `def total= amount`. The `=` has to sit against the name, which is what tells a writer from the endless definition `def total = amount`
- A writer answers wherever it was written: `def self.name=` on a class or module, and `def obj.name=` on one object, both run when an assignment names them
- A `case` clause may name a constant from the top level, as `when ::String`, in both the `when` and `in` forms
- A global variable and a constant of the same name are two different names: `$DEBUG` never answers for a program's own `DEBUG`
- `yield` with no block behind it raises `LocalJumpError`, the way Ruby names a jump with nowhere to land
- `Object#methods` reports what an included or prepended module supplies, walking the same chain a call travels
- Indexing an object that defines no `[]` reaches its `method_missing`, the same as any other name it carries no method for
- A `**` double splat reaches a call written without parentheses, as `take 1, **held`
- A protected method takes an explicit receiver from inside the class that defines it, which is what tells it apart from a private one. A call refused from outside names the marking that refused it
- A `when` clause naming a constant that stands for a value compares against that value, and one naming a class asks what the value is
- `%e` and `%E` write a number in exponent notation, with one digit before the point and a signed two-digit power
- `:%` and `obj.%` name the method, so a `%` written against a symbol's colon or a call's dot is that name rather than the opening of a literal
- `String#to_i` carries digits too wide for a machine word rather than answering zero, and a number that wide matches the `Integer` pattern
- Reopening `NilClass`, `TrueClass`, `FalseClass`, or `Symbol` adds a method those values answer to
- A backslash before something that opens no escape stands for the character alone in a double-quoted string, so `"a\{b"` names three characters. A single-quoted string keeps the backslash
- A parse error inside `eval` names the file it was given, in the `file:line: message` form Ruby writes
- **Binding**: a binding names the locals in force where it was taken, and `local_variables`, `local_variable_get`, `local_variable_set`, and `local_variable_defined?` read and write them. `eval` runs code there and leaves what it binds behind, `source_location` says where the binding was taken, and `dup` and `clone` give a copy written to on its own
- A writer written in a `class << self` body answers an assignment, the same as one written with `def self.name=`
- A `when` clause may name a constant through the module holding it, as `when Socket::SOCK_STREAM`
- `File#flush` sends on what was written through a handle, and `File.rename` and `File.chmod` move a name and set its permissions
- `"%s" % nil` renders as nothing at all, since `%s` asks for `to_s`
- `Object#inspect` shows the instance variables alongside the class and address, each rendered as `inspect` would. An `instance_variables_to_inspect` method chooses which to show, nil from it means all of them, and anything else raises TypeError
- A format string reads `%{name}` and `%<name>` from the Hash it was given, raising KeyError for a name that Hash does not carry. With `$VERBOSE` on, arguments the format never reached are pointed out, and a keyword Hash is not counted among them
- `format` and `sprintf` are private instance methods of Kernel, and `Kernel.format` names the same one
- `fork` splits the process: the child answers nil, or runs the block and exits with its status, while the parent answers the child's process id. Every thread from the parent is marked finished in the child
- `Process.wait`, `.waitpid`, `.wait2`, `.waitpid2`, and `.waitall` wait for a child and record `$?`. `Process.exit`, `.exit!`, and `.abort` end the process the way the bare forms do
- `Thread.current` answers the main thread outside any thread block, and `Thread#kill` marks a thread finished so `alive?` reports false
- `exit` reads its status from an Integer, a boolean, a truncated Float, or anything answering `to_int`, and refuses the rest with TypeError. `exit!` skips the `at_exit` handlers and any `ensure` clause
- `at_exit` handlers run before an uncaught exception is reported, so a handler calling `exit!` replaces both the report and the status
- `abort`, `exit`, and `exit!` are reachable with an explicit receiver on any object, which is what a class makes public with `public :exit`
- `self` at the top level is `main`, rather than an undefined name
- `exec` replaces the process with the command, so nothing after it runs. A command with nothing for the shell to do is run directly, so a missing program raises `Errno::ENOENT` rather than becoming the shell's own exit status. `Kernel.exec` names the same method, and it is one of Kernel's private instance methods
- `def foo(...)` collects every argument and `foo(...)` passes them all on, alongside the bare `*`, `**`, and `&` forwarding forms. A `...` with an operand after it is still a beginless range
- A heredoc opens with a bare `<<TERMINATOR` as well as `<<-` and `<<~`, wherever a value is expected, so `<<TEXT.upcase` reads the heredoc and calls on it while `array << value` stays the shovel operator
- A lambda literal takes part in the expression around it, so `-> { 5 }.call == 5` compares rather than stopping at the call
- `clone` copies the singleton class along with the object, so a method defined on the original answers on the copy. It carries the original's frozen state, or whatever `freeze:` names, and refuses any other value for it with ArgumentError. `initialize_clone` is called with the keyword it was given
- A method that declares no keyword parameters counts a trailing keyword hash as an ordinary positional argument, so passing one to a single-parameter method raises ArgumentError
- `super` inside a singleton method reaches the class's own copy of that method
- `IO.popen` takes an argv Array as readily as a command String, and its handle can be written to as well as read: what is written reaches the child when the input is closed, and the child is waited for once the block returns
- The `-n` flag runs the program once for each line of standard input, with the line in `$_`
- `chomp` and `chop` with no receiver rewrite `$_` in place, `chomp` taking its separator from `$/`. `Kernel.chomp` and `Kernel.chop` do the same, and both are private methods of Kernel
- `Kernel.private_method_defined?` reports the Kernel methods that live in the native dispatch tables rather than in a method map
- `caller` reports each frame as `file:line:in 'label'`, taking the same start and length or Range that `caller_locations` does. A block is named by the scope holding it, so a block written at top level reads `block in <main>`
- `puts` writes an Array a line per element, however deeply nested, and a line of its own for an empty one. A string already ending in a newline is not given a second
- A block's body belongs to the file it was written in, so a backtrace entry for a call made from it names that file wherever the block is called from
- `caller_locations` takes a start and length or a Range, including endless, beginless, and negative-ended ones. Omitting more locations than there are answers nil and omitting exactly as many answers an empty Array. Level 0 is the line the call sits on, each location names the file its frame was called from, and `caller_locations` is one of Kernel's private instance methods
- A backtick literal runs its command through the shell and answers what it wrote to stdout, interpolating the way a double-quoted string does. The command's stderr passes through, `$?` reports how it ended, and a command the shell cannot find raises `Errno::ENOENT`. `Kernel.\`` and the `:\`` symbol name the same method
- `Process::Status` is a constant on Process, answering `stopped?`, `stopsig`, and `pid` alongside its other readers
- `Encoding.default_external` is remembered and reported, and `Encoding::SHIFT_JIS` is defined
- `String#b`
- `autoload` and `autoload?` are reachable by name, registering on Object where top-level constants live, and are listed among Kernel's private instance methods
- A file reached by `load` runs at top level, so a `def` in it belongs to Object rather than to whatever class or module body called the load
- `at_exit` registers a handler and answers it, raising ArgumentError when given no block. Handlers run in reverse order of registration once the program is over, however it ends, and one registered inside a handler runs right after it. A handler still runs when an earlier one raised, `exit` inside one settles the status, and `$!` there is the exception that ended the program
- `rescue => $global` binds the rescued exception to a global variable
- `$?` answers the status of the last child process waited for
- `-r` and `-I` accept their value attached, as `-rfoo`, and `-r` takes an absolute or `./`-relative path outright
- `__dir__` answers the real directory holding the running file, expanding a relative script path, and nil where no file stands behind the code. `eval` with a filename reports that file's directory, and eval through a binding reports nil
- `Dir.chdir` changes the working directory, restoring the previous one after a block and answering what the block returned
- `__FILE__` reports the path the main script was named by on the command line, and constant source locations record the same spelling
- `File.expand_path` expands a relative base against the working directory, so its answer is always absolute
- `Kernel#Float` reads a String strictly: a sign, `_` only between digits, an optional fraction and `e` exponent, and the `0x` hexadecimal form with a `p` binary exponent. Bad text raises ArgumentError, nil raises TypeError, a Complex with an imaginary part raises RangeError, and `exception: false` answers nil
- `Float#nan?`, `#finite?`, and `#infinite?`, which answers the sign of an infinity and nil otherwise
- A Float renders with its fractional part, so `1.0` reads as "1.0" rather than "1"
- Two infinities of the same sign are equal, and a NaN is identical to itself through `equal?` while equal to nothing through `==`
- `Complex` answers `real`, `imaginary`, `to_s`, `inspect`, and `==`, where a complex with no imaginary part equals the plain number it holds. `Complex.polar` and `Complex.rectangular` build one from either pair of components
- `Kernel#Complex` reads a String literal in every form Ruby accepts: integers, fractions, floats, scientific notation, the `i`/`I`/`j`/`J` units, `a+bi`, `m@a` polar form, and `_` digit separators. Bad text raises ArgumentError, a non-number raises TypeError, and `exception: false` answers nil instead
- An Integer equals the Float holding the same value, so `1 == 1.0` is true, in an Array or Hash comparison as much as on its own
- `defined?` on a method call answers nil when the receiver does not answer to that method, rather than reporting every call as a method
- `Encoding::CompatibilityError`, `Encoding::UndefinedConversionError`, and `Encoding::InvalidByteSequenceError` are defined, under StandardError
- `Kernel#Array` puts its argument through `to_ary` and then `to_a`, either of which may be private. One answering nil moves on to the next, one answering a non-Array raises TypeError, and an argument with neither is wrapped in a one-element Array
- A lambda literal is a receiver like any other, so `-> value { value * 2 }.call(3)` chains onto it, with or without parentheses around the parameters and in the `do` form
- `ruby2_keywords` raises NameError for a name no method answers to, and warns rather than applying when the method takes keywords or has no bare `*args` splat
- `refine` takes a module as readily as a class, requires a block, and registers the refinement before the block runs, so calls inside it and every sibling refinement in the same module are already in force
- `Hash#map` and `#collect` yield each key and value and answer an Array of what the block returned
- `Hash.new(default)` answers that value for a key it has no entry for, and `#default`, `#default=`, `#default_proc`, and `#default_proc=` read and set it
- `Hash[...]` builds a hash from another hash, from an array of pairs, or from an even number of key and value arguments, and `Hash[a: 1]` and `Hash[1 => 2]` write the pairs in the brackets
- Hash gained `#empty?`, `#has_value?`/`#value?`, `#key`, `#each_key`, `#each_value`, `#invert`, `#store`, `#clear`, `#compact`, `#except`, `#slice`, `#values_at`, `#fetch_values`, `#transform_keys`, `#transform_values`, `#select`/`#filter`, `#reject`, `#keep_if`, `#delete_if`, `#assoc`, `#rassoc`, `#flatten`, `#sort`, `#shift`, `#deconstruct_keys`, `#any?`, `#none?`, `#all?`, and `#to_hash`
- `<`, `<=`, `>`, and `>=` between hashes compare by containment, asking a non-Hash operand for `to_hash`
- Hash gained `#merge!`/`#update`, `#replace`, `#compact!`, `#select!`/`#filter!`, `#reject!`, `#transform_keys!`, `#transform_values!`, `#to_h`, and `#[]`/`#[]=` as methods of their own
- A key that is not a primitive is matched the way Ruby matches one: same `hash`, then `eql?` asked of the key being looked up
- `Hash#fetch` and `#fetch_values` raise a KeyError that reports the hash and the key, and a block wins over a default value with a warning
- `compare_by_identity` and `compare_by_identity?` record the setting, and a hash derived by `slice`, `merge`, `select`, and the rest carries it over
- `Hash#default(key)` runs the default proc for that key, and setting a default value clears the default proc
- `Hash#each` yields one `[key, value]` array, which a block of two parameters spreads across them, and answers an Enumerator without a block
- A String hash key that reads back as a number, a boolean, or nil keeps its own identity, so `{"1" => x}` has a String key
- A subclass of Hash holds real entries, so `MyHash.new[key] = value` reaches the hash the instance is backed by
- A subclass of Hash or Array compares equal to the plain value it holds, and `to_h` on one answers a plain Hash
- `\xNN` names a byte, a run of them spells one character, and `\uXXXX`, `\u{...}`, `\s`, `\a`, `\b`, `\f`, and `\v` are read in a double-quoted string
- `:"a#{b}"` names a Symbol built at run time rather than the String its characters were assembled in
- Symbol answers a Symbol from `#upcase`, `#downcase`, `#capitalize`, `#swapcase`, `#succ`, and `#next`, gained `#id2name`, `#name`, and `#intern`, and mixes in Comparable
- `String#succ` bumps the rightmost alphanumeric character and carries left, growing the string when the leftmost one wraps, and `#capitalize` and `#swapcase` were added
- `String#to_f` reads an exponent and treats an underscore as a digit separator, and `#start_with?`/`#end_with?` put an argument through `to_str`, with `start_with?` also taking a Regexp
- `Integer#chr` names the character a code point stands for
- Range gained `#to_s`, `#inspect`, `#==`, `#eql?`, and `#count`, walks a String or Symbol range with `succ`, walks two single ASCII characters by code point, and refuses to collect an endless one
- `Range.new(first, last, exclusive)` builds the range a literal would
- Range gained `#first(n)`, `#last(n)`, `#min`, `#max`, `#minmax`, `#cover?`, `#overlap?`, `#size`, `#reverse_each`, and `#to_set`, and `#include?`/`#member?` walk a range of names while every other kind compares against the ends
- A block takes a `|(a, b)|` group, which spreads one array argument across the names, and a `|name:|` keyword parameter with a default
- `-> &b { }` and `-> **rest { }` name a lambda's block and keyword arguments, and a `&name` parameter takes the block the call was handed
- A keyword names a keyword argument, so `parameters(lambda: true)` reads as one
- `Method#name` and `#original_name` answer Symbols, `#receiver` answers what the method is bound to, and `method(:+)` on a number hands out a Method for the operator
- `Integer#upto` and `#downto` answer an Enumerator without a block, and Enumerator gained `#next_values` and `#peek_values`
- `String#lstrip`, `#rstrip`, `#each_line`, and `#lines` with a separator; `Array#to_set` and `Set[...]`
- `p (1..3).to_a` passes what the parentheses hold along with the calls that follow, the way Ruby reads a spaced parenthesis after a method name
- `{nil: 1}` and `{false: 2}` name the symbols `:nil` and `:false`, `{a:, b:}` takes each value from the name itself, and an assignment works as an array element, a ternary branch, or a parenthesized expression
- A Float hash key keeps its fraction, so `4` and `4.0` are different keys
- `Array#[]=` takes an index, a start and a length, or a Range, growing the array with nil when the index is past the end and putting a replacement through `to_ary`
- `Array#[]` and `#slice` coerce an index through `to_int` and raise RangeError for one too large for a machine word
- `Range.new(first, last, exclusive)` builds the range a literal would, and a subclass of Range builds one too
- `values[start, length] = a, b, c` assigns the array the right-hand list builds, `x.[]=(i, v)` names the writer, and `Array[1, 2, 3 => 4]` gathers trailing pairs into a Hash
- `private :hash` names a method Object answers natively, which needs no definition of its own
- `String#dump` renders a string as source that reads back as itself, escaping control and non-ASCII characters and the `#` that would start an interpolation
- Interpolating `nil` adds nothing, since `nil.to_s` is the empty string
- A prepended module's visibility is the one in force for the methods it supplies, so `private :name` on the class does not restrict a public method the prepended module defines under that name
- `prepend` puts a module ahead of the class in the ancestry, so its methods shadow the class's own and `super` from one reaches the class's copy. Constants, `ancestors`, `is_a?`, `constants`, and `singleton_method` all read the prepended module in that position, and a module prepended in one place still appears again where a superclass or an include already carried it
- `alias_method` records the aliasing class as the alias's owner, even when the method it copies came from a prepended or included module
- `Array#join` joins a nested array with the same separator, so `["a", ["b", "c"]].join(" ")` is `"a b c"`, however deeply they nest
- The line-reading command line options: `-n` runs the program once per line of standard input with the line in `$_`, `-p` does the same and writes `$_` out after each pass, `-a` splits each line into `$F`, `-F` names the pattern it splits on, and `-0` names the line separator `$/` reads by, by its octal code. Which of them were written reads back under the flag's own name, so `-a` is `$-a`, and `-w` sets `$VERBOSE`. Short flags cluster and a value rides on the end of the one that takes it, so `-naF:` is `-n -a -F:`
- `$FILENAME` names the file ARGF is reading and `$.` counts the lines read, both following ARGF as it walks the files it was handed. A stream read to the end is closed and cannot be put back to the start
- A source file that opens with a byte order mark is read past it, since the mark names the encoding rather than anything the program says
- `Dir.mkdir` takes the mode the directory is created with, which is where a sticky or setgid bit comes from
- A String carries the encoding it says it is in. `force_encoding` changes that tag without touching what the string holds, and every reference to the same string sees the change. A string cut from another is in the same encoding, so slicing, `upcase`, and a piece taken from a match all keep the source's tag. `pack` answers a run of bytes tagged ASCII-8BIT, an empty format answers US-ASCII, and `String#b` answers a binary copy. `encode` tags a copy of text that is nothing but ASCII, since that reads the same in every ASCII-compatible encoding; text that is not needs a conversion metorex does not carry out, and the copy keeps the encoding it had
- Two Strings holding the same text are two objects. `equal?` tells them apart, and each answers an id of its own, handed out the first time one is asked for and the same every time after
- The values that write themselves each answer one unchanging string: `nil.to_s`, `true.to_s`, `false.to_s`, `Symbol#name`, and `Module#name` hand back the same object every time, where `Symbol#id2name` hands back a fresh one. A module that gains a name answers the new one
- `max_by`, `min_by`, and `minmax_by` answer the value that came first when two tie, which sorting alone does not promise
- `Base64.decode64` reads leniently, passing over anything that is not a base64 character and decoding a short group as far as it goes, where `strict_decode64` refuses both
- `Array#pack` and `String#unpack` share one directive machine, so a directive means the same thing whichever way the bytes travel. Between them they read and write text (`a`, `A`, `Z`), bits and nibbles (`b`, `B`, `h`, `H`), the integer widths (`c`, `C`, `s`, `S`, `i`, `I`, `l`, `L`, `q`, `Q`, `j`, `J`, `n`, `N`, `v`, `V`), floats (`f`, `d`, `e`, `E`, `g`, `G`), code points (`U`), BER integers (`w`), base64, quoted printable, uuencoding, and the placement directives (`x`, `X`, `@`). A count or `*` follows a directive, `!` and `_` ask for the platform's own width, and `<` and `>` name the byte order. `String#unpack1` answers the first value alone
- `?\n`, `?\001`, and `?\x41` name a character by an escape, reading the same escapes a string literal does
- `TracePoint` calls a handler as the interpreter runs. It is built over the events it cares about, switched on around a block or on its own, and handed itself when one of those events happens. `:line` fires once per line of the program, and `:call` and `:return` fire around a method written in Ruby, carrying `event`, `lineno`, `path`, `self`, `method_id`, `callee_id`, `defined_class`, and `return_value`. Reading one of those outside a handler is refused, a handler never fires events of its own, and the core library metorex loads at startup is passed over the way Ruby passes over its C code
- `Integer#upto` and `#downto` without a block answer an Enumerator whatever the endpoint is. Only asking that Enumerator for its size reports that it cannot count to an endpoint it does not understand
- `Marshal::MAJOR_VERSION` and `Marshal::MINOR_VERSION` name the format version
- A closed file handle refuses every reading and every report of where it stands, raising IOError. It still answers `closed?`, `path`, `to_io`, and `inspect`, and `reopen` puts it back to work on another name. `gets` counts the line it hands back, which `lineno` reports and `rewind` resets
- `Matrix::LUPDecomposition` factors a matrix into a lower triangle, an upper triangle, and the permutation the pivoting made, so `l * u == p * a`. It answers `l`, `u`, `p`, `to_a`, `determinant`, `singular?`, and `solve` for a Matrix or a Vector, and `Matrix#lup` builds one
- `Queue` and `SizedQueue` close: `close` marks the queue closed and answers itself, `closed?` reports it, and pushing to a closed queue raises ClosedQueueError. Neither can be frozen, since a queue is the state its own methods change. Both are named under `Thread` as well as at the top level, and the two names reach the same class
- `GC.enable`, `GC.disable`, `GC.stress`, `GC.auto_compact`, and `GC.measure_total_time` read back the way they were written, and `GC::Profiler` switches on and off. Metorex frees an object when its last reference goes, so none of them changes what runs
- `Time.iso8601` and `Time.rfc2822` read the fraction of a second exactly, so `.52` is 13/25 rather than the nearest Float. A leap second written as `:60` names the moment the next second begins, and RFC 822's zone names (`EST`, `EDT`, `CST`, `PDT`, and the rest) carry their offsets. Folding whitespace may sit either side of the colons
- `Math.lgamma` answers Infinity at every pole of the gamma function, which is every whole number at or below zero, with the sign alternating from one pole to the next
- A negative Float raised to a power that is not a whole number answers the principal complex root, and a Float takes a Rational exponent by reading it as a Float
- `File.read` on a directory raises Errno::EISDIR, `File.readlink` on a missing name raises Errno::ENOENT, and `File.mkfifo` reads a name through `to_path` and reports the errno the system gave
- `Complex#coerce` takes a real number to pair with, so a String is refused even though `Complex("2")` reads one, and so is a Numeric that answers false to `real?`
- A Hash subclass keeps its class through `merge`, where the rest of Hash's table answers a plain Hash
- `ThreadGroup` holds a set of threads, `Kernel#syscall` and `Kernel#set_trace_func` are private methods that report themselves unimplemented, `Thread.allocate` is refused, and `include` and `prepend` on a Refinement raise TypeError
- `ObjectSpace._id2ref` warns that it is deprecated, and answers only the values whose id is derived from the value itself
- `ARGF` reads a list of files as though they were one. `ARGF.class.new(a, b)` builds a stream over the names given, and the global `ARGF` reads `ARGV`. It answers `gets`, `read`, `readlines`, `getc`, `readchar`, `each_line`, `file`, `filename`, `path`, `argv`, `lineno`, `pos`, `tell`, `rewind`, `skip`, `eof?`, and `closed?`. `eof?` reports the file being read rather than the whole list, and a stream whose last file has been read to the end is closed, so `pos` refuses with ArgumentError and `eof?` with IOError
- An open file handle reads a character at a time with `getc` and `readchar`, reports and moves its position with `pos`, `tell`, `pos=`, `seek`, and `rewind`, counts lines through `lineno` and `lineno=`, points at another name with `reopen`, and reports `closed?` after `close`. `readchar` at the end raises EOFError
- `File` descends from `IO`, and both include `Enumerable` and `File::Constants`. `File::NULL` names the path that discards what is written to it, and `File.mkfifo` makes a named pipe. `File#stat`, `#lstat`, `#path`, `#to_path`, `#to_io`, and `#birthtime` read the handle's own name, and `File.lchmod` changes a symlink's mode rather than that of what it points at
- `Encoding` names 40 encodings. Two constants for one encoding, such as `BINARY` and `ASCII_8BIT`, reach the same object, so `Encoding.find` on the name an encoding reports answers that same encoding. `Encoding.list` reports each one once, `#inspect` renders `#<Encoding:UTF-8>`, and a dummy encoding such as `ISO-2022-JP` says so through `#dummy?` and in its inspect
- `Process::Tms` carries the four processor-time readings, `Process.warmup` answers true, and `Process.maxgroups` reads and writes the supplementary group ceiling. A setter a module answers natively, such as `Process.maxgroups=`, is reached by the assignment form as well as by `send`
- `GC.count` and `GC.total_time` climb as collections are asked for, `GC#garbage_collect` answers nil, and `GC::Profiler` reports itself switched off with an empty result
- `ThreadGroup` holds a set of threads, `ThreadGroup::Default` is the group a thread starts in, and `Thread#group` reports the one a thread belongs to. An enclosed group refuses to give its threads up
- `and` and `or` bind more loosely than a paren-less call's arguments, so `check x and fallback` calls `check x` and tests what it answered, where `check x && fallback` passes the whole test as the argument
- A bare `yield` may be followed by an operator that can only sit between two operands, so `while yield == :retry` compares what the block answered
- An operator written on a core class, or on a module prepended to one, answers the syntax form: `1 + 2` reaches a `+` defined on Integer or on a module prepended to it, and `super` from that definition reaches the built-in arithmetic. An operator inherited from an ancestor does not shadow the built-in one
- `super` with nothing above the defining class raises NoMethodError rather than a plain error
- `Array#take` and `Array#drop` split an array at a count, raising ArgumentError on a negative one
- `IO.popen` runs a command through the shell and hands back a handle answering `read`, `pid`, `close`, and `closed?`, in the block form too. `err: [:child, :out]` folds the child's stderr into what `read` returns
- `Process.last_status` answers a `Process::Status` for the last child waited for, reading back through `exited?`, `exitstatus`, `signaled?`, `termsig`, `success?`, and `to_i`
- Reopening `Module` or `Class` adds a method every class and module answers, which `respond_to?` reports too. A `const_added` defined that way is the hook for each of them
- `const_added` fires for a top-level class, module, or constant, with Object as the receiver, and only the first time the constant is defined
- `exit` raises SystemExit rather than ending the process outright, so an `ensure` block runs and a `rescue SystemExit` sees it, reading the code through `#status` and `#success?`. `exit!` ends the process immediately. An uncaught SystemExit exits with the status it carries
- A rescue clause places an exception by its class chain, which reaches a namespaced or anonymous subclass that has no name to look up
- `expr; rescue` inside `begin ... end` opens a rescue clause. Only a `rescue` directly following an expression is the modifier form
- `eval` and `parse` for runtime code execution and AST inspection
- `get_source` for runtime method introspection
- AST Inspection API: `Method#body`, `Block#statements`, node type/property access
- DSL Examples: test framework, HTML builder, query builder, configuration language
- Array Methods: `length`/`size`, `push`/`pop`, `shift`/`unshift`, `sort`, `reverse`, `map`, `select`/`filter`, `reduce`, `each`, `join`

### Phase 2: Bytecode VM
- Bytecode compiler (complete: expression, statement, control flow, function/method, class, closure, block compilation, optimization passes)
- Stack-based VM (complete: structure, call frames, execution loop, basic instructions, variables, control flow, function calls, closures, classes/objects, collections)
- Trait/interface system
- Advanced reflection

### Phase 3: Production Ready
- JIT compilation (LLVM)
- Full concurrency (threads, channels, atomics)
- Networking library (HTTP, WebSocket, TCP/UDP)
- Cryptography library
- Optional type system
- Documentation generator
- LSP support

### Phase 4: Advanced Features
- Macro system
- Algebraic data types (Option, Result)
- Functional programming features
- WebAssembly compilation
- Security and sandboxing
- Advanced tooling (profilers, static analysis)

## Contributing

METOREX is in active development. We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone the repository (including Ruby spec submodules)
git clone https://github.com/gdonald/metorex.git
cd metorex
git submodule update --init

# Build the project
cargo build

# Run tests
cargo test

# Run the REPL
cargo run

# Run a Metorex file
cargo run -- my_script.rb

# Discover and run test files in a directory
cargo run -- --test tests/

# Install code coverage tool (required for cargo tarpaulin)
cargo install cargo-tarpaulin

# Run code coverage
cargo tarpaulin --out Stdout

# Run Ruby spec suite (requires submodules)
scripts/run_ruby_spec.sh
```

## License

See [LICENSE](LICENSE) for details.

## Why METOREX?

**For DSL Creators**: Build domain-specific languages naturally with first-class AST access.

**For Scripters**: Ruby-like syntax with powerful built-in libraries.

**For Systems Programmers**: Rust-based VM with performance and safety guarantees.

**For Functional Enthusiasts**: Optional algebraic data types, immutable structures, and functional patterns.

**For Pragmatists**: One language that adapts to your needs - from quick scripts to production systems.

**METOREX: Where meta-programming meets production-ready performance.**
