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

The pcap object represents the overall pcap object read from a pcap file.
It can be used to access the global header elements of a pcap file.

The following table lists the properties available for the pcap object.

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

The pcap packet represents an individual packet read from the pcap file.
Note that this packet has a packet header in front of the ethernet packet.

The following table lists the properties available for the pcap packet object.

| Name | Description |
|------|-------------|
| sec | An integer property representing the timestamp in seconds |
| msec | An integer property representing the timestamp in microseconds or nanoseconds |
| nsec | An integer property representing the timestamp in microseconds or nanoseconds |
| caplen | An integer property representing the capture length |
| wirelen | An integer property representing the length of packet on wire |
| eth | The ethernet object contained within the packet |
| payload | The ethernet object contained within the packet |


## The ethernet object

The ethernet object represents an ethernet packet.
The following table lists the properties available for the ethernet object.

| Name | Description |
|------|-------------|
| src | An string representation of the source mac address |
| dst | An string representation of the destination mac address |
| type | An integer property representing ethertype |
| vlan | An vlan object if the ethertype is 0x8100 |

## The vlan object

The ethernet object represents a vlan packet.
The following table lists the properties available for the vlan object.

| Name | Description |
|------|-------------|
| id | An integer property representing the vlan ID |
| priority | An integer property representing priority code point |
| dei | A boolean property representing DEI |
| vlan | An vlan object if the ethertype is 0x8100 |
