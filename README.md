# shortcut-catapult
Tiny, modular service that lets you create smart shortcuts for browsers with support for placeholders

## How it works
In your browser shortcuts, you create shortcuts with placeholders (commonly `%s`). As the URL for these shortcuts, you configure the HTTP endpoint of `shortcut-catapult` and include the placeholder somewhere in the URL (e.g., as a query parameter).
`shortcut-catapult` then parses the URL and responds with a temporary redirect 302 response according to its configuration.

## Usage

`shortcut-catapult` runs as an HTTP service, listening on localhost. 
```bash
# Run HTTP server
shortcut-catapult daemon --port 8081

# Test the configuration against a single URL (writes redirect URL to stdout or exits with code 2)
shortcut-catapult apply [-|URL]

# Common command line options
--help         print help
--debug        enable DEBUG logging
--info         enable INFO logging
--config FILE  use this sepecific config file

# Environment variables
SHORTCUT_CATAPULT_CONFIG_HOME=$XDG_CONFIG_HOME/shortcut-catapult
XDG_CONFIG_HOME as per https://specifications.freedesktop.org/basedir-spec/latest
CATAPULT_LOG=warning to enable fine-grained logging
```

It reads its configuration from [`$XDG_CONFIG_HOME/shortcut-catapult/config.yml`](https://specifications.freedesktop.org/basedir-spec/latest/).
If the file doesn't exist, an empty skeleton will be created. The file gets re-read on change.

## Configuration
The configuration of `shortcut-catapult` is essentially a tree structure of "modules" that match on parts of the URL.
The top-level `match` key is the entry point. Its value is a matcher. Some matchers recursively specify other matchers.
Other matchers specify a `url`. Matchers decide whether they accept a certain input URL. Matchers that have a `url` configured
and accept an input URL will result in a redirect response to that URL.

The individual matchers are documented below.

### Exact Matcher
An object with a key called `exact`. This matcher matches if there is an exact match for the input URL. 
The example below would match a request to `/armadillo`.

```yaml
match:
  exact: "Armadillo"
  case-sensitive: false # default
  trim: true # default
  url: https://google.com?q=$1
```

You can use the placeholder `$1` to include the matched part of the URL in the redirect URL.

### List Matcher
A list. This matcher consists of a list of sub-matchers. It ask each sub-matcher in order and picks the first that
accepts the input URL. If none of the sub-matchers match, the entire list matcher doesn't accept the URL.

```yaml
match:
- exact: "Elephant"
  url: https://kagi.com?q=Elephant
- exact: "Lion"
  url: https://bing.com?q=Lion
```

### Prefix Matcher
An object with a key called `prefix`. This matcher can be used to form matcher hierarchies distinguished by prefix
or to match a URL directly.

```yaml
match:
  prefix: Arm
  case-sensitive: false # default
  url: https://wikipedia.org
```

You would commonly use this matcher together with the list matcher
to support multiple browser shortcuts with different behavior. In the example below, we use a list matcher
at the top level and prefix matchers as their sub matchers. The URL that the sub-matchers receive will 
have the prefix stripped away. So if you use this config with a URL `/animals/bear`, then the sub-matcher
will receive `bear` as its input. 

```
match:
- prefix: animals/
  match: ... # any sub-matcher
- prefix: cooking/
  match: ... # any sub-matcher
```

Placeholders:
- `$1` the matched prefix
- `$2` the rest of the URL (after the prefix)

### Fuzzy Matcher
Object with the key `fuzzy`. This matcher allows the input to be off by a couple of characters.
```yaml
match:
  fuzzy: Elephant
  tolerance: 3 # default (number of tolerated edits)
  url: https://heavy.animal
```

Placeholders:
- `$1` the matched URL

### Regex Matcher
Object with the key `regex`. This matcher uses a regular expression to match (and extract parts of) the URL.
You can either use it to return a redirect URL directly or with a sub-matcher.

```yaml
match:
  regex: (\w+)\.txt$
  case-sensitive: false # default
  url: https://file.drive/$1.txt
```

Placeholders are the numbered regular expression capture groups.

You can use the `match-with` setting to customize the part of the URL that gets forwarded to the sub-matcher. 
By default, it is whatever the regular expression matched (which doesn't have to be the entire URL).
In the example below, the input URL `/animals/Bears.pdf` would get matched by the regex matcher, which would
then forward `Bea.pdf` to the sub-matcher.
```yaml
match:
  regex: ^animals/(\w{1,3})\w*\.(\w+)
  match-with: $1.$2
  match: ... # any sub-matcher
```
