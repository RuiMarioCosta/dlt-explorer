# 5 Protocol specification

## 5.1 Message format

For both, debug data and control information, the same Dlt message format is used.  
It consists of a "Base Header", an optional "Extension Header", and a Payload
segment.

[PRS_Dlt_01000] The "Base Header" and the "Extension Header" shall always use
the network byte order.

Note: "Network Byte Order" equals "Big Endian".

### 5.1.1 Base Header

[PRS_Dlt_01001] The Base Header shall always consist of the following fields in
the following order:

- Byte 0 – 3: HTYP2 (Header Type for protocol version "2")
- Byte 4: MCNT (Message Counter)
- Byte 5 – 6: LEN (Message Length)

The following fields of the Base Header are only contained if certain conditions are
met. The conditions are defined later in this sub-chapter.

[PRS_Dlt_01002] In addition, the Base Header shall also conditionally consist of the
following fields in the following order:

- MSIN (Message Info) length; 1 byte;
- NOAR (Number of arguments) length; 1 byte;
- TMSP2 (Timestamp version "2") length; 9 bytes;
- MSID (Message ID) length; 4 bytes;

Each element shall be added after the last existing element in the Base Header.
There shall be no gap in between.

Note: Since the above elements are conditional, an absolute byte position can't be
given here, as they may shift due to the activation/deactivation of those conditional
fields (see above).

[PRS_Dlt_01003] If the Log and Trace message is a Data Message in Verbose
Mode or a Control Message, the MSIN (Message Info) and NOAR (Number of
arguments) shall be added to the Base Header.

Note: The information, whether the Log and Trace message is a Data Message in
Verbose or Non-Verbose Mode or a Control Message, is located in the HTYP2 – field
sub-element "CNTI" (Content Info); see the following subchapter for more details.

[PRS_Dlt_01004] If the Log and Trace message is a Data Message (Verbose Mode
or Non-Verbose Mode), the TMSP2 (Timestamp) with a nanosecond resolution shall
be added to the Base Header.

[PRS_Dlt_01005] If the Log and Trace message is a Non-Verbose Mode Data
Message, the MSID (Message ID) shall be added to the Base Header.

#### 5.1.1.1 Header Type  

The "Header Type"-field for protocol version '2' (HTYP2) contains general information
about the Log and Trace message.
Except for the three bits "Version Number" information, all other flags are used to
indicate conditional or optional later content in this Log and Trace message.
In this context here,

- "conditional" means, the required usage is specified in this document.  
- "optional" means, the usage is application specific.

[PRS_Dlt_01006] The Header Type - field (HTYP2) shall be the first element of any
Log and Trace message.

[PRS_Dlt_01007] The size of the Header Type - field (HTYP2) shall be 32 bit.

[PRS_Dlt_01008] The Header Type (HTYP2) shall contain the following information
and shall be encoded in the following way:

- Bit 0 – 1: CNTI (Content Information)
- Bit 2: WEID (With ECU ID)
- Bit 3: WACID (With App- and Context ID)
- Bit 4: WSID (With Session ID)
- Bit 5-7: VERS (Version Number)
- Bit 8: WSFLN (With Source File Name and Line Number)
- Bit 9: WTGS (With Tags)
- Bit 10: WPVL (With Privacy Level)
- Bit 11 WSGM (With Segmentation)
- Bit 12 - 31: reserved (reserved by AUTOSAR for future usage)

[PRS_Dlt_01009] The two "CNTI"-bits (Content Info; bits 0 – 1 in HTYP2) shall be a
2-bit unsigned integer and shall be encoded in the following way:

- 0x0: Verbose Mode Data Message;
- 0x1: Non-Verbose Mode Data Message;
- 0x2: Control Message;
- 0x3: reserved;

[PRS_Dlt_01010] The "VERS"-bits (Version Number; bits 5 – 7 in HTYP2) shall be
a 3-bit unsigned integer and shall contain the Log and Trace protocol version as
defined by AUTOSAR. The version number valid for this specification release is "2".

Note: The "VERS"-bits are located at the same position like in version "1" of the
protocol. Therefore the receivers can always distinguish the protocol versions.

[PRS_Dlt_01011] If one of the following bits are set, the "Extension Header" shall
be added after the "Base Header":

- Bit 2: WEID (With ECU ID)
- Bit 3: WACID (With App- and Context ID)
- Bit 4: WSID (With Session ID)
- Bit 8: WSFLN (With Source File Name and Line Number)
- Bit 9: WTGS (With Tags)
- Bit 10: WPVL (With Privacy Level)
- Bit 11: WSGM (With Segmentation)

Note: The details about the "Extension Header" and the correlation with the above
mentioned bits are specified in a later subchapter.  
Also, the bits 12 – 31 (currently "reserved by AUTOSAR for future usage") are
intended to require the "Extension Header" in the future.

#### 5.1.1.2 Message Counter  

The Message Counter (MCNT) counts Dlt messages transmitted to a selected Log
Channel. Each Log Channel needs to maintain its own Message Counter. On the
receiver side, the Message Counter value can be evaluated to identify lost messages
to a certain level.

[PRS_Dlt_00319] The Message Counter is an unsigned 8-bit (0-255) integer.

[PRS_Dlt_00613] After initialization of the Dlt module, the Message Counter
(MCNT) shall be set to ‘0’.

[PRS_Dlt_00105] The Message Counter shall be incremented by one for each Dlt
message that is transmitted to assigned LogChannel.

[PRS_Dlt_00106] If the Message Counter reaches 255, the counter shall wrap
around and start with the value ‘0’ at the next Log and Trace message to be
transmitted.

#### 5.1.1.3 Message Length

[PRS_Dlt_00320] The Message Length (LEN) field for the complete Log and Trace
message in the Base Header shall be a 16-bit unsigned integer.
[PRS_Dlt_00614] The Message Length (LEN) field in the Base Header shall be set
to the overall length in bytes of the complete Log and Trace message, which is the
sum of:

- the length in bytes of the Base Header itself,  
- the length in bytes of the optional Extension Header and  
- the length in bytes of the optional Payload.

Note: This Message Length (LEN) contains the length of a single simple
LogAndTraceMessage and is independent from any segmentation functionality, as
specified later on (compare chapter "5.1.2.7 Optional Message Segmentation").
Therefore, the upper limit of a single simple LogAndTraceMessage is either limited
by the underlying communication protocol / -medium or by the max.value of the LEN
field (16bit): 65535.

#### 5.1.1.4 Conditional "Message Info"

Like specified above (refer PRS_Dlt_01003), the MSIN (Message Info) is added to
the Base Header in case the Log and Trace message is a Data Message in Verbose
Mode or a Control Message, otherwise the MSIN is not part of the Base Header.

[PRS_Dlt_00618] The Message Info field (MSIN) shall contain the following
information in the following order:

- Bit 0: reserved (reserved)
- Bit 1-3: MSTP (Message Type)
- Bit 4-7: MTIN (Message Type Info)

[PRS_Dlt_00324] The Message Type (MSTP) shall be a 3-bit unsigned integer.

[PRS_Dlt_00120] The Message Type (MSTP) shall have one of the following
values:

- 0x0: DLT_TYPE_LOG (Dlt Log Message)
- 0x1: DLT_TYPE_APP_TRACE (Dlt Trace Message)
- 0x2: DLT_TYPE_NW_TRACE (Dlt Network Message)
- 0x3: DLT_TYPE_CONTROL (Dlt Control Message)
- 0x4 – 0x7:  Reserved

[PRS_Dlt_00325] The Message Type Info field (MTIN) shall be a 4-bit unsigned
integer.

[PRS_Dlt_00619] If the MSTP field is set to 0x0 (i.e. Dlt Log Message), the
Message Type Info field (MTIN) shall have one of the following values with the
following meaning:

- 0x1:  DLT_LOG_FATAL (Fatal system error)
- 0x2: DLT_LOG_DLT_ERROR (Application error)
- 0x3: DLT_LOG_WARN (Correct behavior cannot be ensured)
- 0x4: DLT_LOGINFO (Message of LogLevel type “Information”)
- 0x5: DLT_LOG_DEBUG (Message of LogLevel type “Debug”)
- 0x6: DLT_LOG_VERBOSE (Message of LogLevel type “Verbose”)
- 0x7 – 0xF: Reserved

[PRS_Dlt_00620] If the MSTP field is set to 0x1 (i.e. Dlt Trace Message), the
Message Type Info field (MTIN) shall have one of the following values with the
following meaning:

- 0x1: DLT_TRACE_VARIABLE (Value of variable)
- 0x2: DLT_TRACE_FUNCTION_IN (Call of a function)
- 0x3: DLT_TRACE_FUNCTION_OUT (Return of a function)
- 0x4: DLT_TRACE_STATE (State of a State Machine)
- 0x5: DLT_TRACE_VFB (RTE events)
- 0x6 – 0xF: Reserved

[PRS_Dlt_00621] If the MSTP field is set to 0x2 (i.e. Dlt Network Message), the
Message Type Info field (MTIN) shall have one of the following values with the
following meaning:

- 0x1: DLT_NW_TRACE_IPC (Inter-Process-Communication)
- 0x2: DLT_NW_TRACE_CAN (CAN Communications bus)
- 0x3: DLT_NW_TRACE_FLEXRAY (FlexRay Communications bus)
- 0x4: DLT_NW_TRACE_MOST (Most Communications bus)
- 0x5: DLT_NW_TRACE_ETHERNET (Ethernet Communications bus)
- 0x6: DLT_NW_TRACE_SOMEIP (Inter-SOME/IP Communication)
- 0x7-0xF: User Defined (User defined settings)

[PRS_Dlt_00622] If the MSTP field is set to 0x3 (i.e. Dlt Control Message), the
Message Type Info field (MTIN) shall have one of the following values with the
following meaning:

- 0x1:  DLT_CONTROL_REQUEST (Request Control Message)
- 0x2: DLT_CONTROL_RESPONSE (Respond Control Message)
- 0x3-0xF: Reserved

#### 5.1.1.5 Conditional "Number of Arguments"

Like specified above (refer PRS_Dlt_01003), the NOAR (Number of Arguments) is
added to the Base Header in case the Log and Trace message is a Data Message in
Verbose Mode or a Control Message. Otherwise the NOAR is not part or the Base
Header.

Number of Arguments represents the number of consecutive parameters or the
number of consecutive control commands in the payload segment of one Dlt
message.

[PRS_Dlt_00326] The Number of Arguments field (NOAR) shall be an 8-bit
unsigned integer.

[PRS_Dlt_00126] The Number of Arguments field (NOAR) shall contain the number
of provided arguments or control commands within the payload.

#### 5.1.1.6 Conditional "ns-Timestamp"

Like specified above (refer PRS_Dlt_01004), the TMSP2 (ns-Timestamp) is added to
the Base Header in case the Log and Trace message is a Data (Verbose Mode or
Non-Verbose Mode), otherwise the TMSP2 is not part of the Base Header.
The conditional Timestamp is used to add timing information on when a Dlt message
has been generated.

[PRS_Dlt_01012] Format of ns-Timestamp ⌈The length for the ns-timestamp shall
be 9 byte:

- The lower 4 byte / uint32 shall be the nanoseconds part of the timestamp.
- The upper 5 byte / 40 bits shall be the second's part of the timestamp.

The time shall start from 1970-01-01, 00:00:00,00000, i.e. this timestamp shall be
derived from an absolute / global time that has a Synchronized Time Base.⌋
(RS_LT_00017)

Note:
0 to 1.099.511.627.776s ~ 34.841 years
0 to 999999999ns [0x3B9A C9FF];  
Invalid value in nanoseconds: [0x3B9A CA00] to [0x3FFF FFFF];  
Bit 30 and 31 are reserved in this case.

[PRS_Dlt_01013] **Format of ns-Timestamp for ECUs without a synchronized
time base** If a specific ECU can't provide an absolute time starting from 1970-01-01,
00:00:00,00000 time, the bit 31 in the nanoseconds field shall be set and the time
shall start from the ECU startup.

[PRS_Dlt_01014] **Substance of the ns-Timestamp** The ns-Timestamp value shall
hold the time at the moment an LT User calls the LT module and hands over its LT
content.

#### 5.1.1.7 Conditional "Message ID"

Like specified above (refer PRS_Dlt_01005), the MSID (Message ID) is added to the
Base Header in case the Log and Trace message is a Data Message in Non
Verbose Mode, otherwise the MSID is not part of the Base Header.

[PRS_Dlt_00624] ⌈The Message ID shall be a 32-bit unsigned integer.

Note: More details can be found in subchapter 5.1.3.1 Payload in Non-Verbose
Mode.

### 5.1.2 Extension Header  

The Extension Header contains additional data that facilitates the interpretation of the
pure LT content. Thus, further properties of the LT content, such as the exact origin,
are transmitted here.

In case one of the following bits of the "HTYP2"-field in the Base Header are set to
‘1’, additional information is transmitted which are defined in the Extension Header
format:

- Bit 2: WEID (With ECU ID)
- Bit 3: WACID (With App- and Context ID)
- Bit 4: WSID (With Session ID)
- Bit 8: WSFLN (With Source File Name and Line Number)
- Bit 9: WTGS (With Tags)
- Bit 10: WPVL (With Privacy Level)
- Bit 11: WSGM (With Segmentation)

The basic design principles for the Extension Header are:

- All of its fields are optional and therefore the complete Extension Header is
optional.
- Whether a specific field needs to be added to the Extension Header is
indicated by the above mentioned bits ("flags") from the "HTYP2"-field in the
Base Header.
- The order of the fields in the Extension Header is defined by the order of the
corresponding flags in the "HTYP2"-field from the Base Header.
- A field consist of a length specifier and the value itself (there are a few
exceptions to this).

The length information for a specific field can also be '0'. In this case, no field value is
provided and the field ends after the length byte.

In order to allow for future expansions of the Extension Header without breaking
backward compatibility, all further fields in the future must start with a 1 byte length
information. In this way, an implementation according to the current specification can
always move from field to field (and thus finally also to the end of the header), even if
it can't interpret all of the field values.  
Future field elements in the Extension Header are enabled by using the reserved
flags in the "HTYP2"-field from the Base Header: currently bits 12 – 31 (marked as
"reserved by AUTOSAR for future usage").

As a consequence for all new fields in the future / for all currently "reserved" flags:
the number of flags in "HTYP2" which are set to "1" have to be equal with the number
of length information added to the Extension Header.

[PRS_Dlt_01015] **Locate Extension Header after Base Header** If the Extension
Header gets used, it shall be directly attached after the Base Header fields.

[PRS_Dlt_01016] **Sequence of the fields in the Extension Header** The fields are
to be optionally added to the Extension Header depending on and in the sequence of
the corresponding flags in the "HTYP2"-field from the Base Header.

#### 5.1.2.1 Optional ECU-ID

The optional ECU ID is used to identify which ECU has sent a Log and Trace
message. Therefore, it is highly recommended that the ECU ID is unique within the
vehicle.

[PRS_Dlt_01017] **Possibility to send the ECU ID** If the bit 2 (WEID, "With ECU
ID") in the "HTYP2"-field of the Base Header is set, the LT-message shall contain the
length byte and the string value for the ECU ID, added to the Extension Header.

[PRS_Dlt_01018] **Length information** The length byte shall be the first byte in the
ECU ID field and shall count the number of characters used for the ECU ID.

[PRS_Dlt_01019] **ECU ID format** The coding of the ECU ID shall contain only
ASCII characters without a special terminating item like the NUL-character (\0) at the
end.

Note: The string end is only given by the Length information for the ECU-ID.

#### 5.1.2.2 Optional Application ID and Context ID

The Application ID is an abbreviation of the application which generates the Dlt
message.  

The Context ID is a user defined ID to (logically) group Dlt messages generated by
an application.

[PRS_Dlt_01020] **Possibility to send the Application ID and Context ID** If the bit
3 (WACID, "With App- and Context ID") in the "HTYP2"-field of the Base Header is
set, the LT-message shall contain the length bytes and the string values for the
Application ID and the Context ID, added to the Extension Header.

[PRS_Dlt_01021] **Sequence of Application ID and Context ID** If the Application
ID and the Context ID are added to the Extension Header, the Application ID field
shall be the first and the Context ID field shall be the second.

[PRS_Dlt_01022] **Length information of Application ID and Context ID** For each
of the two fields (Application ID and Context ID) the length byte shall be the first byte
in that ID field and shall count the number of characters used for that ID.

[PRS_Dlt_01023] **Application ID and Context ID format** The coding of the
Application ID and Context ID shall contain only ASCII characters without a special
terminating item like the NUL-character (\0).

Note: The string ends for Application ID and Context ID are only given by the length
specification which precedes each.

[PRS_Dlt_01054]  "#" (U+0023) as prefix of Context IDs shall be reserved for
modelled Log and Trace messages standardized by AUTOSAR.

[PRS_Dlt_01055] Context IDs for Log and Trace messages defined by stack
vendors shall have a "+" (U+002B) prefix, followed by the vendor's numerical
identifier converted to a string as per PRS_Dlt_01056, followed by a vendor-defined
remainder.

[PRS_Dlt_01056] 16-bit vendor-IDs are converted to a 2-char ASCII string using
Base62 encoding using the string
"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
as digit sequence.

Note: The highest Vendor ID that can be encoded with Base62 in two characters
without data loss is 3843 (0x0f03). This ID will be encoded as the string "zz".

Example: Using the Vendor ID 0x07bb, the context ID starts with the string "+Vv" with
"Vv" being the Base62-encoded string for 0x07bb.

#### 5.1.2.3 Optional Session ID

The optional Session ID is used to identify the source of a log or trace message
within an ECU.  

[PRS_Dlt_01024] If the bit 4 (WSID, "With Session ID") in the "HTYP2"-field of the
Base Header is set, the LT-message shall contain the Session ID, added to the
Extension Header.

Note: Since the Session ID is defined to be of 32-bit length, this Session ID field in
the Extension Header does NOT have an extra length byte in it.

[PRS_Dlt_00322] The Session ID field shall be a 32-bit unsigned integer.

#### 5.1.2.4 Optional Source File Name and Source Line Number

To identify the source of log or trace content some information to find the location in
the source code shall be added to a Log and Trace message.  

Therefore:

- the name of the source file (string) and -
- the line number in the source file (unsigned integer)
can be added to the Extension Header.

In a more general way, the source file name is also called "source file identifier": A
"source file identifier" constitutes a means to identify the source code file in which a
log message originates. That would typically be a filename or filename stem, but
could also be a full (or relative) path, or even an entirely different kind, e.g. a hash
sum in case filenames are considered to be sensitive data.

[PRS_Dlt_01025] **Possibility to send the source file identifier and the source
line number** If the bit 8 (WSFLN: "With Source File Name and Line Number ") in the
"HTYP2"-field of the Base Header is set, the LT-message shall contain the length
byte for the source file identifier string and the string value itself and additionally the
source line number where the LT message originates from, added to the Extension
Header.

[PRS_Dlt_01026] **Content in the Extension Header for the source file identifier
and the source line number** If the source file identifier and the source line number
are transmitted in the Extension Header, the following sequence shall be used:

- the length byte for the source file identifier string;
- the string value itself for the source file identifier string;
- the source line number

Note: since the source line number is defined to have 32 bit, no additional length byte
for the source line number is contained.

[PRS_Dlt_01027] **Definition of the length information** The field for the length
shall count the number of bytes which the source file identifier consumes. This
number also equals the amount of UTF-8 code units.

[PRS_Dlt_01028] **Source file identifier format** The coding of the source file
identifier shall be with UTF-8 code units without BOM and without termination
characters.

[PRS_Dlt_01029] **Substance of the source file identifier** The source file identifier
shall contain the indication from where the log or trace content originates.

Note: This indication can be made up by the filename stem (filename without an
extension) and maybe additionally the filename extension and/or the path (full or
partial) to the file can be included.

Alternatively, in case the origin of the log and trace content is considered to be
sensitive data, the source file identifier can also be something else, like a hash sum
or any other encoded identification.

Note: Up to 255 bytes respectively UTF-8 code units can be used.  

[PRS_Dlt_01030] **Source Line Number format** The length for the source line
number shall be four bytes interpreted as a 32-bit unsigned integer.

Note: The Source Line Number starts counting with '1', i.e. the value '0' is not used.
Since the length for the line number is statically defined as a 32-bit unsigned integer,
no separate length byte shall be added to the Extension Header.

#### 5.1.2.5 Optional Tags

For avoiding bus traffic, especially when logging with Verbose Mode and for tracing,
tags could help the application or functional cluster to classify the messages more
finely by topic.

[PRS_Dlt_01031] **Possibility to send tags for filtering purposes** If the bit 9
(WTGS: "With Tags") in the "HTYP2"-field of the Base Header is set, the LT
message shall contain the following elements in the given sequence:  

- the number of attached tags (NOTG);
- for each attached tag:
  - a length byte for the tag name string
  - the string value for the tag name
added to the Extension Header.

[PRS_Dlt_01032] **Definition of the Number of tags** The field "NOTG" (Number of
Tags) shall be an 8 bit unsigned integer value and shall count the tags to follow in the
Extension Header. Therefore at maximum 255 tags can be added to a LT-message.

[PRS_Dlt_01033] **Definition of the length information for each tag** The field for
the length shall count the number of bytes which the tag name consumes.

[PRS_Dlt_01034] **Tag name format** The coding of the tag name shall be with
ASCII characters without a special terminating item like the NUL-character (\0).

Note: The string end is only given by the Length information for the tag name.

#### 5.1.2.6 Optional Privacy Level

The Privacy Level helps to identify the Log and Trace content towards the degree of
privacy to it. Logging clients, no matter if in the ECU or outside of the ECU, have the
possibility to consider the privacy level at the Log and Trace message to ensure
intended and allowed processing of them.  

[PRS_Dlt_01035] **Possibility to add a privacy level for the containing Log and
Trace message** If the bit 10 (WPVL: "With Privacy Level") in the "HTYP2"-field of
the Base Header is set, the LT-message shall contain the value for the privacy level
of the current LT-message, added to the Extension Header.

[PRS_Dlt_01036] **Format of the Privacy Level** The length for the Privacy Level
shall be one byte unsigned integer.

Note: Since the length of the Privacy Level is defined to be one byte, no extra length
information is added in the Privacy Level field of the Extension Header.

Note: There is no global definition for the meaning of each single value number of the
Privacy Level.

Note: It is up to the external viewer tool or any other instance that interpret or forward
the message, to meet this privacy request.

#### 5.1.2.7 Optional Message Segmentation Information

Message Segmentation can be used to transfer a larger amount of payload data that
otherwise would have not fit into a single simple LT message. Remember: the total
length of a normal, single simple LT message is either limited by the underlying
communication protocol / -medium or by the max.value of its "LEN" field in the
BaseHeader (16-bit unsigned integer): 65535. In both cases, the available remaining
size for the payload is smaller, because the message headers need to be included as
well.

[PRS_Dlt_01043] **Criteria to use Message Segmentation**  Based on the
knowledge of the lower layer frame length limit or the limit of the "LEN" field in the
BaseHeader, the L&T module shall decide whether segmentation needs to be used
or not.

Note: Segmentation should not be used for smaller amounts of payload data, that
also fit into a single simple LT message.

[PRS_Dlt_01044] **Indication of Message Segmentation** If Message Segmentation
is used, the bit 11 (WSGM, "With Segmentation") in the "HTYP2"-field of the
Base Header shall be set and the LT-message shall contain the Segmentation
Information, added to the Extension Header.

[PRS_Dlt_01045] **Content of the Segmentation-Information in the Extension Header**
The Segmentation-Information shall contain the following elements in the given
sequence:

- the length byte for this Segmentation-Information in bytes;
- 8-bit FrameType, which can either be  
  - 0 := "FirstFrame";
  - 1 := "ConsecutiveFrame";
  - 2 := "LastFrame";
  - 3 := "AbortFrame";
- x-bit Segmentation details, depending on FrameType:  
  - "FirstFrame":  
    - 64-bit unsigned integer "TotalLength";
  - "ConsecutiveFrame":  
    - 32-bit unsigned integer "SequenceCounter;";
  - "LastFrame":  
    - 0-bit: n/a; no segmentation details;
  - "AbortFrame":  
    - 8-bit unsigned integer "AbortReason";  
    - 0 - no error/no reason
    - communication time out
    - 2 - insufficient resources
    - 3 - sequence/protocol error

[PRS_Dlt_01046] **FrameType sequence for transmission of a Segmented
Message**
The segmentation sequence shall be:

- 1 "FirstFrame";
- 0 ... 4.294.967.295 "ConsecutiveFrames": depending on the TotalLength of
the segmented data.
- 1 "LastFrame";

Note: "FirstFrame" and "ConsecutiveFrame" use the maximum available size a of
regular LT message. The "LastFrame" can be shorter.

[PRS_Dlt_01048] **Aborting the sequence**
After the "FirstFrame" but before the "LastFrame", there can be an "AbortFrame" to
stop the sequence in case a problem occurred. The already transmitted parts shall
be discarded. After an "AbortFrame" was sent, the next allowed FrameType is a
"FirstFrame".  

[PRS_Dlt_01049] **Content of the "TotalLength" information**
The "TotalLength" information shall contain the overall payload data size in bytes that
needs to be transmitted in a segmented way.

[PRS_Dlt_01050] **Usage of the segmentation "SequenceCounter"**
The segmentation SequenceCounter shall only be used in the
"ConsecutiveFrame(s)", in case there are any. After each FirstFrame, the
SequenceCounter shall start with '0' and can get at maximum '4.294.967.295' in the
last "ConsecutiveFrame" before the "LastFrame". There shall be no wrap-around
('4.294.967.295' -> '0', '1' ... ).

[PRS_Dlt_01051] **Transfer of Payload data blocks**
In case FrameType equals {FirstFrame or ConsecutiveFrame or LastFrame}, the
Payload-segment of the LT-messages shall be sequentially filled with data blocks as
slices from the overall user-data. The sequence of the data slices must be in line with
the transmitted LT-messages:
(with: FirstFrame: FF; ConsecutiveFrame: CF; LastFrame: LF; SequenceCounter: SqCntr)
FF [DataSlice_0], CF_0 [SqCntr = 0; DataSlice1], CF_1 [, SqCntr = 1; DataSlice2],
CF_<n> [SqCntr = <n>; DataSlice<n+1>], LF [DataSlice<n+2>].

### 5.1.3 Body/Payload format
