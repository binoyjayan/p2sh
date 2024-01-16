# Filters

This is formal introduction of filters, please refer to the tutorial for
concrete examples.

## Introduction

Filters are potent programming constructs in p2sh. When present in a
p2sh script, filter statements differ from regular program statements.
They work in conjunction with a pcap stream read from standard input and
a stream written to standard output. To process a pcap file, its content
can be directed into the script via stdin using shell redirection. The
pcap stream can also originate from programs like tcpdump or tshark.

When filter statements appear in a script, the interpreter initially
executes all statements, excluding the filters. It subsequently reads the
pcap stream packet by packet, processing each against the script's filter
statements.

Filters statements start with a '@' character and is of the following form.

```
@ pattern { action }
```

A pattern is an expression that evaluates to a boolean value. If true,
it triggers an action, encapsulated within curly braces. Think of filter
statements as if constructs with a condition and body. Either the pattern
or action can be optional, but not both. Omitting the pattern defaults it
to true, ensuring the action always evaluates. If the action is missing,
the default is to write the current packet to stdout.

Filters resemble constructs found in the AWK language.

## Patterns

Patterns are expressions that evaluate to a boolean value. There is also a
special type of pattern that evaluates to true at the end of the pcap stream.
This pattern is named 'end'. It facilitates summary operations post processing
the pcap stream. Patterns can also make use of special variables (described
below).

### Example patterns

| Name | Description |
|------|-------------|
| NP <  10 | If packet number is less than 10 |
| PL <= 64 | If packet length is <= 64 bytes |
| ($1).type == 0x8100 | If eth.type is 0x8100 |

## Special variables


| Name | Description |
|------|-------------|
| argv | Command-line arguments |
| NP | Number of packets processed so far |
| PL | Captured length of the current packet |
| WL | Length of the current packet on wire |
| TSS | Seconds component of the packet timestamp |
| TSS | Micro or nano seconds component of the packet timestamp |
| $0 | Current pcap packet. Includes pcap packet header |
| $1 | Current ethernet packet |
| $2 | Current ipv4 packet [ if ($1).type is ipv4 ] - TBD |
| $3 | Current udp/tcp/.. packet |
| $4 | Raw data - TBD |
| $n | Packet 'n' level deep |

Note that if the packets are encapsulated, the '$2', '$3' etc can mean
something else. In these cases, use the ether type '($1).type' or
the protocol ('($2).proto') type to determine the inner packet contents.
Refer to the tutorial for example of pattern usage.

## Actions

Actions consist of statements within curly braces, supporting all language
constructs like locals, conditionals, loops, and functions. Filter statements
execute in their scope; thus, variables declared within an action are local
to that filter. However, actions also have access to global variables and
functions defined outside but not within other filters.
