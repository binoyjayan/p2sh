## Filters

## Introduction

Filters are powerful programming constructs in p2sh. For a formal introduction
of filters, please refer to the language reference.

Filters statements start with a '@' character and is of the following form.
The pattern is an expression that evaluates to a true or false. Actions are
statements executed as a consequence.

Filters resemble constructs found in the AWK language.

```
@ pattern { action }
```

Filter statements execute against every pcap packet read in the pcap stream.

## Examples

Note: While using the command mode, use single quotes to pass in the command
instead of a double quotes to avoid cli such as bash intefering with special
symbols such as '$0'.

Note: Advanced examples are in the `examples/filters` directory.

### Match every packet

```bash
p2sh -c '@ true' < in.pcap > out.pcap
```

Here, the default action is to write the pcap. The output pcap file will look
exactly like the input pcap file.

### Match 64 byte packets

```bash
p2sh -c '@ ($0).caplen  <= 64 true' < in.pcap > out.pcap
p2sh -c '@ ($0).wirelen <= 64 true' < in.pcap > out.pcap
```

Here, the only 64 byte packets are matched and only those will end up in the
output pcap file.

### Print packet number and length

This example displays the packet number and its length. Use the '-s'
flag to skip writing the pcap header, as we're only showing packet
number and length within the action.

```bash
p2sh -s -c '@ { eprintln("{}: {}", NP, PL) }' < in.pcap
```

### Print timestamp and length

This example prints the packet timestamp and its length.

```bash
p2sh -s -c '@ { eprintln("{}.{}: {}", ($0).sec, ($0).usec, PL) }' < in.pcap
```

### Protocols at different depth

```bash
p2sh -sc '@ { puts(($0)); }' < in.pcap
p2sh -sc '@ { puts(($1)); }' < in.pcap
p2sh -sc '@ { puts(($2)); }' < in.pcap
```

### Source to destination

```bash
p2sh -sc '@ { eprintln("{} -> {}", ($1).src, ($1).dst);}' < in.pcap
p2sh -sc '@ { eprintln("{} -> {}", ($2).src, ($2).dst);}' < in.pcap
```

### Packets from a specific source

```bash
p2sh -sc '@ ($2).src == "192.168.29.58" { puts(($2).src, " -> ", ($2).dst); }' < in.pcap
```

### Interact with other programs

Display the packet timestamp and length but use the input from another program.

```bash
tcpdump -i eth0 -w - | p2sh -s -c '@ { eprintln("{}.{}: {}", ($0).sec, ($0).usec, PL) }'
```

### Print summary info

Save this in a file named `script.p2`

```bash

#!/usr/bin/env p2sh

let cap_size = 0;
let wire_size = 0;

@ { cap_size = cap_size + PL; }
@ { wire_size = wire_size + WL; }
@ end {
  eprintln("Number of packets:  {}", NP);
  eprintln("Total capture size: {}", cap_size);
  eprintln("Total size on wire: {}", wire_size);
}

./script.p2 -s < test.pcap
```

Use the '-s' flag to skip writing the pcap header, as we're displaying
only the packet summary.

