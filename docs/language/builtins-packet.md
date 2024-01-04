# Packet processing Builtins

The following table lists the builtin functions available for packet processing.
The main set of builtin functions can be found [here](./builtins.md)

| Name | Description |
|------|-------------|
| [**pcap_open**](#pcap_open) | Open a pcap file for reading or writing |
| [**pcap_stream**](#pcap_stream) | Open stdin or stdout to read or write pcap stream |
| [**pcap_read_next**](#pcap_read_next) | Read the next packet from a pcap file handle |
| [**pcap_read_all**](#pcap_read_all) | Read all packets from a pcap file handle |
| [**pcap_write**](#pcap_write) | Write a packet to a file or stdout handle |

### Description

### <a name="pcap_open"></a>pcap_open
Open a pcap file for reading or writing.
It accepts a file path and an optional mode as a string and returns
a file handle that can be used for reading the pcap file.
If there is an IO error, `get_errno` can be used to get the last os error
and `strerror` to convert it to a string. This function also returns the
same as an IO error. So, alternatively, `is_error` can be used to check if
the returned value is an error object.

Note that this API supports only the legacy pcap format.

Example:
```
let f = pcap_open("/path/to/file.pcap", "r");
```

### <a name="pcap_stream"></a>pcap_stream
Open stdin or stdout for reading or writing pcap data.
If there is an IO error, `get_errno` can be used to get the last os error
and `strerror` to convert it to a string. This function also returns the
same as an IO error. So, alternatively, `is_error` can be used to check if
the returned value is an error object.

Note that this API supports only the legacy pcap format.

Example:
```
let f1 = pcap_stream(stdin);
let f2 = pcap_stream(stdout);
```

Modes supported
| mode | Description |
|------|-------------|
| r | Open file for reading. Return error if the file does not exist |
| w | Open or create file for writing. Truncate if exists |
| x | Create a file and open it for writing. Return error if it exits |

### <a name="pcap_read_next"></a>pcap_read_next
It accepts a file handle as first argument and returns a packet. Note that
this packet also has pcap header in front of it. To extract the ethernet packet,
use the expression "pkt.eth", where 'pkt' is the packet returned.
For more information on accessing the packet fields, refer to the packet properties.

If there is an IO error, `get_errno` can be used to get the last os error
and `strerror` to convert it to a string. This function also returns the
same as an IO error. So, alternatively, `is_error` can be used to check if
the returned value is an error object.
```
let f = open("test.pcap");
let packet = pcap_read_next(f);
```

### <a name="pcap_read_all"></a>pcap_read_all
It accepts a file handle as first argument and returns all packet as an
array of packets.

If there is an IO error, `get_errno` can be used to get the last os error
and `strerror` to convert it to a string. This function also returns the
same as an IO error. So, alternatively, `is_error` can be used to check if
the returned value is an error object.

```
let f = open("test.pcap");
let packets = pcap_read_all(f);
```

### <a name="pcap_write"></a>pcap_write
Write a packet to a pcap file, stdout or stderr. It accepts a file handle as the
first argument and a packet that has a packet header as the second argument.
It returns the number of bytes written. This includes the size of the ethernet
packet and the pcap packet header.

If there is an IO error, `get_errno` can be used to get the last os error
and `strerror` to convert it to a string. This function also returns the
same as an IO error. So, alternatively, `is_error` can be used to check if
the returned value is an error object.

```
let f = open("test", "w");
pcap_write(f, packet);
```

