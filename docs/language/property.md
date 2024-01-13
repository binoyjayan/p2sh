# Packet property expressions

Packet property expressions maybe used to read packet properties
or to set a packet property, thereby modifying a packet or a pcap.
The property expression looks like the following:

```
<object>.<property>
```

The object can be any packet object such as a pcap, a packet packet,
an ethernet packet etc.

An object property can also be set as:
```
<object>.<property> = value;
```

Note that the properties use language specific data types while accessing.

## Pcap object

This object represents the overall pcap object read from a pcap file.
It can be used to access the global header elements of a pcap file.

The following table lists the properties available for the this object.

| Name | Description |
|------|-------------|
| magic | An integer property representing the pcap magic |
| major | An integer property representing the pcap major version |
| minor | An integer property representing the pcap minor version |
| thiszone | An integer property representing thiszone |
| sigflags |An integer property representing sigflags |
| snaplen | An integer property representing snaplen |
| linktype | An integer property representing linktype |


## Pcap packet object

This object represents an individual packet read from the pcap file.
Note that this packet has a packet header in front of the ethernet packet.

The following table lists the properties available for the this object.

| Name | Description |
|------|-------------|
| sec | An integer property representing the timestamp in seconds |
| msec | An integer property representing the timestamp in microseconds or nanoseconds |
| nsec | An integer property representing the timestamp in microseconds or nanoseconds |
| caplen | An integer property representing the capture length |
| wirelen | An integer property representing the length of packet on wire |
| eth | The ethernet object contained within the packet |
| payload | The ethernet data as an array of bytes |


## The ethernet object

This object represents an ethernet packet.
The following table lists the properties available for the this object.

| Name | Description |
|------|-------------|
| src | An string property representing the source mac address |
| dst | An string property representing the destination mac address |
| type | An integer property representing ethertype |
| vlan | An vlan object if the ethertype is 0x8100 |
| ipv4 | An ipv4 object if the ethertype is 0x0800 |
| payload | The ethernet payload as an array of bytes |

## The vlan object

This object represents a vlan packet.
The following table lists the properties available for the vlan object.

| Name | Description |
|------|-------------|
| id | An integer property representing the vlan ID |
| priority | An integer property representing priority code point |
| dei | A boolean property representing DEI |
| vlan | An vlan object if the ethertype is 0x8100 |
| ipv4 | An ipv4 object if the ethertype is 0x0800 |
| payload | The vlan payload as an array of bytes |

## The ipv4 object

This object represents a ipv4 packet.
The following table lists the properties available for the this object.

| Name | Description |
|------|-------------|
| version | An read only integer property representing the ipv4 version |
| ihl | An integer property representing Initial Header Length |
| totlen | An integer property representing Total ipv4 packet |
| id | An integer property representing packet identifier |
| dscp | An integer property representing differentiated services field |
| ecn | An integer property representing Explicit congestion notification field |
| flags | An integer property representing IP flags |
| fragoff | An integer property representing fragment offset |
| ttl | An integer property representing ttl |
| proto | An integer property representing protocol |
| checksum | An integer property representing checksum |
| src | An string property representing source ip |
| dst | An string property representing destination ip |
| udp | A udp object if the protocol is 17 |
| payload | The ipv4 payload as an array of bytes |

### The udp object

This object represents a udp packet.
The following table lists the properties available for this object.

| Name | Description |
|------|-------------|
| srcport | An string property representing source port |
| dstport | An string property representing destination port |
| len | An integer property representing length udp header and data |
| checksum | An integer property representing checksum of header and data |
| payload | The udp payload as an array of bytes |
