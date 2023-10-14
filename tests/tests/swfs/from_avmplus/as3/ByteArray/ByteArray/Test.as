/* -*- Mode: C++; c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// General principles for this test suite:
//
//  - never write just one, write at least two (to test that position 
//    advances correctly and output is placed correctly)
//  - ditto read
//  - test both little and big endian for multibyte data
//  - test both aligned and unaligned access for multibyte data
//
// Search for "TODO" for comments about missing tests.

    package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import flash.utils.ByteArray
    import flash.errors.EOFError
    import flash.errors.IOError
import com.adobe.test.Assert;

//     var SECTION = "ByteArray";
//     var VERSION = "as3";
//     var TITLE   = "test ByteArray class";


// Utility to make and pad a ByteArray
function makeByteArray(padding=0) : ByteArray
{
    var bytearray:ByteArray=new ByteArray();
    for ( var i=0 ; i < padding ; i++ )
        bytearray.writeByte(0);
    return bytearray;
}

// Utility to test for RangeError
function expectRangeError(tag, thunk)
{
    var exn_ok = "No exn";
    try                   { thunk(); }
    catch (e: RangeError) { exn_ok = "OK"; }
    catch (e)             { exn_ok = "Wrong type"; }
    Assert.expectEq(tag, "OK", exn_ok);
}

// Utility to test for EOFError
function expectEOF(tag, thunk)
{
    var exn_ok = "No exn";
    try                 { thunk(); }
    catch (e: EOFError) { exn_ok = "OK"; }
    catch (e)           { exn_ok = "Wrong type"; }
    Assert.expectEq(tag, "OK", exn_ok);
}

// Utility to test for IOError
function expectIOError(tag, thunk)
{
    var exn_ok = "No exn";
    try                { thunk(); }
    catch (e: IOError) { exn_ok = "OK"; }
    catch (e)          { exn_ok = "Wrong type"; }
    Assert.expectEq(tag, "OK", exn_ok);
}

function testBasicProperties() {
    var bytearray:ByteArray=new ByteArray();

    Assert.expectEq(
      "ByteArray constructor no args",
      true,
      bytearray!=null
      );

    // operations on empty bytearray

    Assert.expectEq(
      "ByteArray length of empty",
      0,
      bytearray.length);

    Assert.expectEq(
      "ByteArray toString empty",
      "",
      bytearray.toString())

    Assert.expectEq(
      "ByteArray available on empty",
      0,
      bytearray.bytesAvailable);

    Assert.expectEq("ByteArray position on empty",
        0,
        bytearray.position);

    Assert.expectEq("ByteArray endianness on empty",
        "bigEndian",
        bytearray.endian);
}

testBasicProperties();

function testSetLengthAndPosition() {
    var bytearray:ByteArray=new ByteArray();

    // Test: setting length to 0 sets position to 0

    bytearray.writeByte(1);
    bytearray.writeByte(2);
    Assert.expectEq("ByteArray trivial length",
        2,
        bytearray.length);
    Assert.expectEq("ByteArray trivial position",
        2,
        bytearray.position);
    bytearray.length = 0;
    Assert.expectEq("ByteArray position after clearing",
        0,
        bytearray.position);

    // Test: setting position beyond length does not update length.

    bytearray.length = 0;
    bytearray.position = 47;
    Assert.expectEq("ByteArray position can exceed length, #1",
        47,
        bytearray.position);
    Assert.expectEq("ByteArray position can exceed length, #2",
        0,
        bytearray.length);

    // Test: writing updates at position > length inserts zero padding and writes
    // byte at the appropriate position.

    bytearray.writeByte(12);

    Assert.expectEq("ByteArray position can exceed length, #3",
        48,
        bytearray.position);
    Assert.expectEq("ByteArray position can exceed length, #4",
        48,
        bytearray.length);
    Assert.expectEq("ByteArray position can exceed length, #5",
        0,
        bytearray[11]);
    Assert.expectEq("ByteArray position can exceed length, #6",
        12,
        bytearray[47]);
}

testSetLengthAndPosition();

function testBoolean() 
{
    var bytearray:ByteArray=makeByteArray();
    bytearray.writeBoolean(true);
    bytearray.writeBoolean(false);
    Assert.expectEq("ByteArray position after writing Booleans",
        2,
        bytearray.position);

    bytearray.position=0;
    Assert.expectEq(
      "ByteArray move position to 0",
      0,
      bytearray.position);

    Assert.expectEq(
      "ByteArray write/read boolean true",
      true,
      bytearray.readBoolean());

    Assert.expectEq(
      "ByteArray write/read boolean false",
      false,
      bytearray.readBoolean());
}

testBoolean();

function testShort() 
{
    // One endianness or the other
    function readShort_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeShort(100);
        bytearray.writeShort(-200);
        bytearray.position=offset;
        Assert.expectEq("ByteArray readShort_1 #1 " + endian,
                    100,
                    bytearray.readShort());
        Assert.expectEq( "ByteArray readShort_1 #2 " + endian,
                     -200,
                     bytearray.readShort());
        Assert.expectEq("ByteArray readShort_1 #3" + endian,
                    4+offset,
                    bytearray.position);
    }

    // Mixed endianness: write with one, read with the other
    function readShort_tests2(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = "bigEndian";
        bytearray.position=offset;
        bytearray.writeShort(int(0x1234));
        bytearray.writeShort(int(0xFEDC));
        bytearray.position=offset;
        bytearray.endian = "littleEndian";
        Assert.expectEq("ByteArray readShort_2 #1",
                    int(0x3412),
                    bytearray.readShort());
        Assert.expectEq("ByteArray readShort_2 #2",
                    int(0xFFFFDCFE),   // Sign extended
                    bytearray.readShort());
        Assert.expectEq("ByteArray readShort_2 #3",
                    4+offset,
                    bytearray.position);
    }

    // EOF at various offsets and alignments
    function readShort_tests3(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);  // use offset to create the alignment
        bytearray.writeShort(0x1234);

        for ( var i=0 ; i < 2 ; i++ ) {
            var v;
            expectEOF("ByteArray readShort_3 #1 " + offset + " " + (i+1),
                      (function () {
                          bytearray.position = offset + i + 1;
                          v = bytearray.readShort();
                      }));
        }
    }

    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readShort_tests1("bigEndian", offs);
        readShort_tests1("littleEndian", offs);
        readShort_tests2(offs);
        readShort_tests3(offs);
    }
}

testShort();

function testUnsignedShort() 
{
    // One endianness or the other
    function readUShort_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeShort(100);
        bytearray.writeShort(uint(-200) & 65535);
        bytearray.position=offset;
        Assert.expectEq("ByteArray readUShort_1 #1 " + endian,
                    uint(100),
                    bytearray.readUnsignedShort());
        Assert.expectEq("ByteArray readUShort_1 #2 " + endian,
                    uint(-200) & 65535,
                    bytearray.readUnsignedShort());
        Assert.expectEq("ByteArray readUShort_1 #3" + endian,
                    4+offset,
                    bytearray.position);
    }

    // Mixed endianness: write with one, read with the other
    function readUShort_tests2(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = "bigEndian";
        bytearray.position=offset;
        bytearray.writeShort(uint(0x1234));
        bytearray.writeShort(uint(0xFEDC) & 65535);
        bytearray.position=offset;
        bytearray.endian = "littleEndian";
        Assert.expectEq("ByteArray readUShort_2 #1",
                    uint(0x3412),
                    bytearray.readUnsignedShort());
        Assert.expectEq("ByteArray readUShort_2 #2",
                    uint(0xDCFE),
                    bytearray.readUnsignedShort());
        Assert.expectEq("ByteArray readUShort_2 #3",
                    4+offset,
                    bytearray.position);
    }

    // EOF at various offsets and alignments
    function readUShort_tests3(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);  // use offset to create the alignment
        bytearray.writeShort(0x1234);

        for ( var i=0 ; i < 2 ; i++ ) {
            var v;
            expectEOF("ByteArray readUShort_3 #1 " + offset + " " + (i+1),
                      (function () {
                          bytearray.position = offset + i + 1;
                          v = bytearray.readUnsignedShort();
                      }));
        }
    }

    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readUShort_tests1("bigEndian", offs);
        readUShort_tests1("littleEndian", offs);
        readUShort_tests2(offs);
        readUShort_tests3(offs);
    }
}

testUnsignedShort();

function testInt()
{
    // One endianness or the other
    function readInt_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeInt(100);
        bytearray.writeInt(-200);
        bytearray.position=offset;
        Assert.expectEq("ByteArray readInt_1 #1 " + endian,
                    100,
                    bytearray.readInt());
        Assert.expectEq( "ByteArray readInt_1 #2 " + endian,
                     -200,
                     bytearray.readInt());
        Assert.expectEq("ByteArray readInt_1 #3" + endian,
                    8+offset,
                    bytearray.position);
    }

    // Mixed endianness: write with one, read with the other
    function readInt_tests2(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = "bigEndian";
        bytearray.position=offset;
        bytearray.writeInt(int(0x12345678));
        bytearray.writeInt(int(0xFEDCBA98));
        bytearray.position=offset;
        bytearray.endian = "littleEndian";
        Assert.expectEq("ByteArray readInt_2 #1",
                    int(0x78563412),
                    bytearray.readInt());
        Assert.expectEq("ByteArray readInt_2 #2",
                    int(0x98BADCFE),
                    bytearray.readInt());
        Assert.expectEq("ByteArray readInt_2 #3",
                    8+offset,
                    bytearray.position);
    }

    // EOF at various offsets and alignments
    function readInt_tests3(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);  // use offset to create the alignment
        bytearray.writeInt(0x12345678);

        for ( var i=0 ; i < 4 ; i++ ) {
            var v;
            expectEOF("ByteArray readInt_3 #1 " + offset + " " + (i+1),
                      (function () {
                          bytearray.position = offset + i + 1;
                          v = bytearray.readInt();
                      }));
        }

        // Testing for wraparound problems when reading with position
        // much greater than length.  Given that the bytevector size
        // is 1000, a length of 0xFFFFFFF0 will always wrap around on
        // a 32-bit system.  On a 64-bit system we depend on the C++
        // run-time code using uint32_t to represent length and
        // position; if it did not we might abort due to a too-large
        // allocation attempt.

        bytearray.length = 1000;
        expectEOF("ByteArray readInt_3 #2 at position=2^32-16",
                  (function () {
                      bytearray.position = 0xFFFFFFF0;
                      v = bytearray.readInt();
                    }));
    }

    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readInt_tests1("bigEndian", offs);
        readInt_tests1("littleEndian", offs);
        readInt_tests2(offs);
        readInt_tests3(offs);
    }
}

testInt();
   
function testUnsignedInt()
{
    // One endianness or the other
    function readUInt_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeUnsignedInt(100);
        bytearray.writeUnsignedInt(uint(-200));
        bytearray.position=offset;
        Assert.expectEq("ByteArray readUnsignedInt_1 #1 " + endian,
                    100,
                    bytearray.readUnsignedInt());
        Assert.expectEq("ByteArray readUnsignedInt_1 #2 " + endian,
                    uint(-200),
                    bytearray.readUnsignedInt());
        Assert.expectEq("ByteArray readUnsignedInt_1 #3" + endian,
                    8+offset,
                    bytearray.position);
    }

    // Mixed endianness: write with one, read with the other
    function readUInt_tests2(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = "bigEndian";
        bytearray.position=offset;
        bytearray.writeUnsignedInt(uint(0x12345678));
        bytearray.writeUnsignedInt(uint(0xFEDCBA98));
        bytearray.position=offset;
        bytearray.endian = "littleEndian";
        Assert.expectEq("ByteArray readUnsignedInt_2 #1",
                    uint(0x78563412),
                    bytearray.readUnsignedInt());
        Assert.expectEq("ByteArray readUnsignedInt_2 #2",
                    uint(0x98BADCFE),
                    bytearray.readUnsignedInt());
        Assert.expectEq("ByteArray readUnsignedInt_2 #3",
                    8+offset,
                    bytearray.position);
    }

    // EOF at various offsets and alignments
    function readUInt_tests3(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);  // use offset to create the alignment
        bytearray.writeUnsignedInt(0x12345678);

        for ( var i=0 ; i < 4 ; i++ ) {
            var v;
            expectEOF("ByteArray readUInt_3 #1 " + offset + " " + (i+1),
                      (function () {
                          bytearray.position = offset + i + 1;
                          v = bytearray.readInt();
                      }));
        }
    }

    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readUInt_tests1("bigEndian", offs);
        readUInt_tests1("littleEndian", offs);
        readUInt_tests2(offs);
        readUInt_tests3(offs);
    }
}

testUnsignedInt();

function testFloat()
{
    // One endianness or the other
    function readFloat_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeFloat(1.25);
        bytearray.writeFloat(12345.5);
        Assert.expectEq("ByteArray writeFloat_1 #1 " + endian,
                    8+offset,
                    bytearray.position);
        bytearray.position=offset;
        Assert.expectEq("ByteArray readFloat_1 #1 " + endian,
                    1.25,
                    bytearray.readFloat());
        Assert.expectEq("ByteArray readFloat_1 #2 " + endian,
                    12345.5,
                    bytearray.readFloat());
        Assert.expectEq("ByteArray readFloat_1 #3" + endian,
                    8+offset,
                    bytearray.position);
    }

    // Mixed endianness: write with one, read with the other.  
    function readFloat_tests2(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        var temp:ByteArray=new ByteArray;

        bytearray.endian = "bigEndian";
        bytearray.position=offset;

        bytearray.writeFloat(1.25);    // write big
        bytearray.writeFloat(12345.5); //   endian

        bytearray.endian = "littleEndian";
        bytearray.position=offset;

        temp.endian = "littleEndian";

        temp.writeFloat(bytearray.readFloat());  // read little endian
        temp.writeFloat(bytearray.readFloat());  //   and write little endian

        temp.position = 0;
        temp.endian = "bigEndian";
        Assert.expectEq("ByteArray readFloat_2 #1",
                    1.25,
                    temp.readFloat());           // read big endian
        Assert.expectEq("ByteArray readFloat_2 #2",
                    12345.5,
                    temp.readFloat());
    }

    // EOF at various offsets and alignments
    function readFloat_tests3(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);  // use offset to create the alignment
        bytearray.writeFloat(Math.PI);

        for ( var i=0 ; i < 4 ; i++ ) {
            var v;
            expectEOF("ByteArray readFloat_3 #1 " + offset + " " + (i+1),
                      (function () {
                          bytearray.position = offset + i + 1;
                          v = bytearray.readFloat();
                      }));
        }
    }

    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readFloat_tests1("bigEndian", offs);
        readFloat_tests1("littleEndian", offs);
        readFloat_tests2(offs);
        readFloat_tests3(offs);
    }
}

testFloat();

function testDouble() 
{
    // One endianness or the other
    function readDouble_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeDouble(1.25);
        bytearray.writeDouble(12345.5);
        Assert.expectEq("ByteArray writeDouble_1 #1 " + endian,
                    16+offset,
                    bytearray.position);
        bytearray.position=offset;
        Assert.expectEq("ByteArray readDouble_1 #1 " + endian,
                    1.25,
                    bytearray.readDouble());
        Assert.expectEq("ByteArray readDouble_1 #2 " + endian,
                    12345.5,
                    bytearray.readDouble());
        Assert.expectEq("ByteArray readDouble_1 #3" + endian,
                    16+offset,
                    bytearray.position);
    }

    // Mixed endianness: write with one, read with the other.  
    function readDouble_tests2(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        var temp:ByteArray=new ByteArray;

        bytearray.endian = "bigEndian";
        bytearray.position=offset;

        bytearray.writeDouble(1.25);    // write big
        bytearray.writeDouble(12345.5); //   endian

        bytearray.endian = "littleEndian";
        bytearray.position=offset;

        temp.endian = "littleEndian";

        temp.writeDouble(bytearray.readDouble());  // read little endian
        temp.writeDouble(bytearray.readDouble());  //   and write little endian

        temp.position = 0;
        temp.endian = "bigEndian";
        Assert.expectEq("ByteArray readDouble_2 #1",
                    1.25,
                    temp.readDouble());           // read big endian
        Assert.expectEq("ByteArray readDouble_2 #2",
                    12345.5,
                    temp.readDouble());
    }

    // EOF at various offsets and alignments
    function readDouble_tests3(offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);  // use offset to create the alignment
        bytearray.writeDouble(Math.PI);

        for ( var i=0 ; i < 8 ; i++ ) {
            var v;
            expectEOF("ByteArray readDouble_3 #1 " + offset + " " + (i+1),
                      (function () {
                          bytearray.position = offset + i + 1;
                          v = bytearray.readDouble();
                      }));
        }
    }

    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readDouble_tests1("bigEndian", offs);
        readDouble_tests1("littleEndian", offs);
        readDouble_tests2(offs);
        readDouble_tests3(offs);
    }
}

testDouble();

function testByte() 
{
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeByte(-257);
    bytearray.writeByte(37);
    Assert.expectEq("testByte: ByteArray position",
                2,
                bytearray.position);
    Assert.expectEq("testByte: ByteArray length",
                2,
                bytearray.length);
    bytearray.position=0;
    Assert.expectEq( "ByteArray readByte",
                 -1,
                 bytearray.readByte());
    Assert.expectEq( "ByteArray readByte",
                 37,
                 bytearray.readByte());

    var v;
    expectEOF("ByteArray readByte EOF",
              (function () {
                  bytearray.position = bytearray.length;
                  v = bytearray.readByte();
              }));
}

testByte();

function testUnsignedByte() 
{
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeByte(-259);
    bytearray.writeByte(37);
    Assert.expectEq("testUnsignedByte: ByteArray position",
                2,
                bytearray.position);
    Assert.expectEq("testUnsignedByte: ByteArray length",
                2,
                bytearray.length);
    bytearray.position=0;
    Assert.expectEq( "ByteArray readUnsignedByte",
                 253,
                 bytearray.readUnsignedByte());
    Assert.expectEq( "ByteArray readUnsignedByte",
                 37,
                 bytearray.readUnsignedByte());

    var v;
    expectEOF("ByteArray readUnsignedByte EOF",
              (function () {
                  bytearray.position = bytearray.length;
                  v = bytearray.readUnsignedByte();
              }));
}

testUnsignedByte();

function testUtf() 
{
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeUTF("string");
    Assert.expectEq(
        "ByteArray position of utf string",
        8,
        bytearray.position);
    bytearray.position=0;
    Assert.expectEq(
        "ByteArray length of utf string",
        8,
        bytearray.length);
    Assert.expectEq(
        "ByteArray readUTF",
        "string",
        bytearray.readUTF());

    // Also see the readUTFBytes case below.
    //
    // This is arguably a bug but it's how it currently behaves (Bugzilla 687341).
    // readUTF will return a string consisting of the characters up to and not including
    // the NUL, but the position will be updated as if the entire string were consumed.
    bytearray.length = 0;
    bytearray.position = 0;
    bytearray.endian = "bigEndian";
    bytearray.writeByte(0);
    bytearray.writeByte(4);
    bytearray.writeByte(108);
    bytearray.writeByte(97);
    bytearray.writeByte(0);
    bytearray.writeByte(115);

    bytearray.position = 0;
    Assert.expectEq("ByteArray readUTF on contents containing NUL: contents",
                "la",
                bytearray.readUTF());
    Assert.expectEq("ByteArray readUTF on contents containing NUL: position",
                6,
                bytearray.position);

    // Test EOF in data area
    expectEOF("ReadUTF EOF in content",
              (function () {
                  bytearray.length = 0;
                  bytearray.endian = "bigEndian";
                  bytearray.writeUTF("super");
                  bytearray[1] = 6; // One too much
                  bytearray.position = 0;
                  bytearray.readUTF();
              }));

    // Test EOF in length area
    expectEOF("ReadUTF EOF in length, #1",
              (function () {
                  bytearray.length = 0;
                  bytearray.readUTF();
              }));

    expectEOF("ReadUTF EOF in length, #2",
              (function () {
                  bytearray.length = 0;
                  bytearray.writeByte(0);
                  bytearray.position = 0;
                  bytearray.readUTF();
              }));

    // Doc sez: A RangeError will be thrown for writeUTF if the string length exceeds 65535.
    expectRangeError("RangeError in writeUTF",
                     (function () {
                         var s = "86868686";
                         while (s.length <= 65535)
                             s = s + s;
                         bytearray.writeUTF(s);
                     }));

    // Skip UTF-8 BOM.
    // This seems fairly ill-defined and ad-hoc since the BOM is skipped but is accounted for in the byte count,
    // but it's what we do, so test that we continue to do it...
    bytearray.length = 0;
    bytearray.position = 0;
    bytearray.endian = "bigEndian";
    bytearray.writeByte(0);
    bytearray.writeByte(6);
    bytearray.writeByte(0xEF);
    bytearray.writeByte(0xBB);
    bytearray.writeByte(0xBF);
    bytearray.writeUTFBytes("string");
    bytearray.position = 0;
    Assert.expectEq("ByteArray readUTF skips UTF8 BOM after length bytes but includes it in the length",
                "str",
                bytearray.readUTF());

    // TODO: test invalid UTF - we should still get data, in a predictable way (invalid input turns into individual bytes)
}

testUtf();

function testUtfBytes() 
{
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeUTFBytes("string");
    bytearray.position=0;
    Assert.expectEq(
        "ByteArray length of utf bytes string",
        6,
        bytearray.length);
    Assert.expectEq(
        "ByteArray readUTFBytes",
        "string",
        bytearray.readUTFBytes(6));

    // Also see the readUTF case above.
    //
    // This is arguably a bug but it's how it currently behaves (Bugzilla 687341).
    // readUTF will return a string consisting of the characters up to and not including
    // the NUL, but the position will be updated as if the entire string were consumed.
    bytearray.length = 0;
    bytearray.position = 0;
    bytearray.endian = "bigEndian";
    bytearray.writeByte(108);
    bytearray.writeByte(97);
    bytearray.writeByte(0);
    bytearray.writeByte(115);

    bytearray.position = 0;
    Assert.expectEq("ByteArray readUTFBytes on contents containing NUL: contents",
                "la",
                bytearray.readUTFBytes(4));
    Assert.expectEq("ByteArray readUTFBytes on contents containing NUL: position",
                4,
                bytearray.position);

    // Test EOF in data area
    expectEOF("ReadUTFBytes EOF in content",
              (function () {
                  bytearray.length = 0;
                  bytearray.endian = "bigEndian";
                  bytearray.writeUTF("super");
                  bytearray.position = 2;
                  bytearray.readUTFBytes(6); // one too much
              }));

    // Skip UTF-8 BOM.
    // This seems fairly ill-defined and ad-hoc since the BOM is skipped but is accounted for in the byte count,
    // but it's what we do, so test that we continue to do it...
    bytearray.length = 0;
    bytearray.position = 0;
    bytearray.endian = "bigEndian";
    bytearray.writeByte(0xEF);
    bytearray.writeByte(0xBB);
    bytearray.writeByte(0xBF);
    bytearray.writeUTFBytes("string");
    bytearray.position = 0;
    Assert.expectEq("ByteArray readUTFBytes skips UTF8 BOM but includes it in the length",
                "str",
                bytearray.readUTFBytes(6));

    // TODO: test invalid UTF - we should still get data, in a predictable way (invalid input turns into individual bytes)
}

testUtfBytes();

function testCompressAndUncompress() {
    var bytearray:ByteArray = makeByteArray();
    bytearray.writeUTFBytes("string");
    bytearray.compress();
    Assert.expectEq(
        "ByteArray length after compress",
        14,
        bytearray.length);

    bytearray.uncompress();
    Assert.expectEq(
        "ByteArray length after uncompress",
        6,
        bytearray.length);

    bytearray.length = 0;
    bytearray.position = 0;
    bytearray.writeUTFBytes("string");
    bytearray.deflate();
    Assert.expectEq(
        "ByteArray length after deflate",
        8,  // This is what the inflate algorithm produces on 2011-09-22, so we accept it as Truth.
        bytearray.length);

    bytearray.inflate();
    Assert.expectEq(
        "ByteArray length after inflate",
        6,
        bytearray.length);

    bytearray.length=0;
    bytearray.compress();
    Assert.expectEq(
        "ByteArray length after empty compress",
        0,
        bytearray.length);

    bytearray.uncompress();
    Assert.expectEq(
        "ByteArray length after empty uncompress",
        0,
        bytearray.length);

    // Bugzilla 691251: ByteArray uncompress and inflate leak memory if presented with invalid data
    // We should get an IOError here (not a problem) and in Debug builds we should not assert on exit.
    bytearray.length = 0;
    bytearray.position = 0;
    bytearray.writeUTFBytes("string");
    bytearray.compress();

    expectIOError("Uncompress on mangled data",
                  (function () {
                      bytearray[0] ^= 0x86;
                      bytearray.uncompress();
                  }));

    // Bugzilla 691251: ByteArray uncompress and inflate leak memory if presented with invalid data
    // We should get an IOError here (not a problem) and in Debug builds we should not assert on exit.
    bytearray.length = 0;
    bytearray.position = 0;
    bytearray.writeUTFBytes("string");
    bytearray.deflate();

    expectIOError("Inflate on mangled data",
                  (function () {
                      bytearray[0] ^= 0x86;
                      bytearray.inflate();
                  }));
}

// https://bugzilla.mozilla.org/show_bug.cgi?id=778727
//testCompressAndUncompress();

function testEndian() {
    var bytearray:ByteArray = makeByteArray();
    Assert.expectEq(
        "get default endian",
        "bigEndian",
        bytearray.endian);

    bytearray.endian="littleEndian";
    Assert.expectEq(
        "set endian littleEndian",
        "littleEndian",
        bytearray.endian);

    bytearray.endian="bigEndian";
    Assert.expectEq(
        "set endian bigEndian",
        "bigEndian",
        bytearray.endian);

    var err="none";
    try {
        bytearray.endian="none";
    } catch (e) {
        err=e.toString();
    }
    Assert.expectEq(
        "exception thrown when endian is to littleEndian or bigEndian",
        "ArgumentError: Error #2008",
        err.substring(0,26));
    Assert.expectEq(
        "endian value is uchanged after invalid set",
        "bigEndian",
        bytearray.endian);
}

testEndian();

function testBracketSyntax() {
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeByte(10);
    bytearray.writeByte(11);
    bytearray.writeByte(12);
    bytearray.position = 0;

    Assert.expectEq(
        "ByteArray get [] syntax",
        12,
        bytearray[2]);
   
    bytearray[2]=13;
    Assert.expectEq(
        "ByteArray set [] syntax",
        13,
        bytearray[2]);

    // We can write negative values but should read positive values
    bytearray[2] = -13;
    Assert.expectEq(
        "ByteArray set [] / get [] syntax",
        243,
        bytearray[2]);

    // This is sad, but it is the traditional behavior: reading
    // outside the range returns undefined, it does not throw or
    // return 0.  Ergo bytearray "byte" reads are not monotyped.
    Assert.expectEq("Bytearray get[] out of range",
                undefined,
                bytearray[3]);

    // When writing out of range, extend the bytearray and zero-fill
    bytearray[4] = 37;

    Assert.expectEq("ByteArray set[] out of range: changed element",
                37,
                bytearray[4]);

    Assert.expectEq("ByteArray set[] out of range: length",
                5,
                bytearray.length);

    Assert.expectEq("ByteArray set[] out of range: zero-fill",
                0,
                bytearray[3]);

    // Sanity: all this reading and writing has not changed the position
    Assert.expectEq("ByteArray get[] and set[]: position",
                0,
                bytearray.position);

    // Sanity: accesses with Atom are correct.  We could have more tests here.
    var v = {}
    v[String.prototype.toLowerCase.call("X")] = 2;  // Defeat most reasonable optimizations
    
    bytearray[v.x] = 42;
    Assert.expectEq("ByteArray set[] with Atom index",
                42,
                bytearray[2]);

    bytearray[2] = 112;
    Assert.expectEq("ByteArray get[] with Atom index",
                112,
                bytearray[v.x]);
}

testBracketSyntax();

function testLengthManipulation() {
    var bytearray:ByteArray = new ByteArray;
    bytearray.length=10;
    Assert.expectEq(
        "ByteArray empty slots filled with 0",
        0,
        bytearray[9]);

    var bytearray_shrink=new ByteArray;
    bytearray_shrink.length=10;
    bytearray_shrink.length=5;
    Assert.expectEq(
        "ByteArray shrink length",
        5,
        bytearray_shrink.length);
}

testLengthManipulation();

function testReadBytes() {
    var bytearray:ByteArray = makeByteArray();
    bytearray.writeUTF("abcdefghijk");
    bytearray.position = 0;

    var bytearray2:ByteArray=new ByteArray;
    bytearray.readBytes(bytearray2,0,0);

    for ( var i="a".charCodeAt(0), k=0 ; i <= "k".charCodeAt(0) ; i++, k++ )
        Assert.expectEq("readBytes correct content",
                    i,
                    bytearray2[k+2]);

    var bytearray3:ByteArray=new ByteArray;
    var pos = bytearray.position;
    bytearray.readBytes(bytearray3,8);
    Assert.expectEq(
        "ByteArray readBytes 8 length copies values, check size",
        8,
        bytearray3.length);
    Assert.expectEq(
        "ByteArray readBytes 8 length copies values, check position",
        pos, // Position *is not* updated by readBytes()
        bytearray.position);

    expectEOF("EOF in readBytes",
              (function () {
                  bytearray.position = 0;
                  bytearray.readBytes(bytearray3, 0, bytearray.length+1);
              }));

    // Doc sez: A RangeError will be thrown if the value of offset+length exceeds 2^32-1
    expectRangeError("RangeError in readBytes",
                     (function () {
                         bytearray3.position = 0;
                         bytearray.readBytes(bytearray3, 0xFFFFFFFF, 1);
                     }));

    // TODO: test more combinations of offset and count
}

testReadBytes();

function testWriteBytes() {
    var bytearray:ByteArray = makeByteArray();
    for ( var i=0 ; i < 10 ; i++ )
        bytearray.writeByte(i);

    var bytearray4=new ByteArray;
    bytearray4.writeBytes(bytearray);
    Assert.expectEq(
        "ByteArray writeBytes: length",
        10,
        bytearray4.length);

    Assert.expectEq(
        "ByteArray writeBytes: position",
        10, // Position *is* updated by writeBytes()
        bytearray4.position);

    for ( var i=0 ; i < 10 ; i++ ) {
        Assert.expectEq(
            "ByteArray writeBytes: content",
            i,
            bytearray4[i]);
    }

    var bytearray5=new ByteArray;
    bytearray5.writeBytes(bytearray,1,5);
    Assert.expectEq(
        "ByteArray writeBytes",
        5,
        bytearray5.length);

    // TODO: test more combinations of offset and count
}

testWriteBytes();

function testHasAtomProperty() {
    var bytearray_atom:ByteArray=new ByteArray;
    bytearray_atom.writeByte(1);
    bytearray_atom.writeByte(2);
    bytearray_atom.writeByte(3);
    Assert.expectEq(
        "ByteArray hasAtomProperty true",
        true,
        1 in bytearray_atom);
    Assert.expectEq(
        "ByteArray hasAtomProperty false",
        false,
        5 in bytearray_atom);
}

testHasAtomProperty();

function testBOM() {
    var bytearray_bom:ByteArray=new ByteArray;

// TODO: toString also skips little-endian and big-endian UTF-16 BOMs (0xFF 0xFE and 0xFE 0xFF).
    bytearray_bom[0]=0xef;
    bytearray_bom[1]=0xbb;
    bytearray_bom[2]=0xbf;
    bytearray_bom[3]=100;
    bytearray_bom[4]=97;
    bytearray_bom[5]=110;
    bytearray_bom[6]=33;
    Assert.expectEq(
        "ByteArray with bom toString",
        "dan!",
        bytearray_bom.toString());

    var bytearray_str:ByteArray=new ByteArray;
    bytearray_str[0]=100;
    bytearray_str[1]=97;
    bytearray_str[2]=110;
    bytearray_str[3]=33;
    Assert.expectEq(
        "ByteArray with no bom toString",
        "dan!",
        bytearray_str.toString());
    
// bad partial sequence
    var bytearray_bad : ByteArray = new ByteArray();
    bytearray_bad[0]=0xE4; // 19968
    bytearray_bad[1]=0xB8;
    bytearray_bad[2]=0x80;
    bytearray_bad[3]=0xE4; // bad sequence
    bytearray_bad[4]=0xE4; // 19968
    bytearray_bad[5]=0xB8;
    bytearray_bad[6]=0x80;
    Assert.expectEq(
        "ByteArray with partial bad utf-8 sequence",
        "\u4e00\u00E4\u4e00",
        bytearray_bad.toString());

// truncated utf-8 sequence
    bytearray_bad = new ByteArray();
    bytearray_bad[0]=0xE4; // truncated sequence
    bytearray_bad[1]=0xB8;
    Assert.expectEq(
        "ByteArray with truncated utf-8 sequence",
        "\u00E4\u00B8",
        bytearray_bad.toString());

// utf-8 sequence > 0x1FFFF
    bytearray_bad = new ByteArray();
    bytearray_bad[0]=0xFB; // character == 0x3FFFF
    bytearray_bad[1]=0xBF;
    bytearray_bad[2]=0xBF;
    bytearray_bad[3]=0xBF;
    bytearray_bad[4]=0xBF;
    bytearray_bad[5]=0xE4; // 19968
    bytearray_bad[6]=0xB8;
    bytearray_bad[7]=0x80;
    Assert.expectEq(
        "ByteArray with out-of-range utf-8 sequence",
        "\udbbf\udfff\u00BF\u4e00",
        bytearray_bad.toString());

// compress/uncompress with BOM
    var bytearray_compress:ByteArray = new ByteArray();
    bytearray_compress[0]=0xef;
    bytearray_compress[1]=0xbb;
    bytearray_compress[2]=0xbf;
    bytearray_compress[3]=100;
    bytearray_compress[4]=97;
    bytearray_compress[5]=110;
    bytearray_compress[6]=33;
    // original length = 7
    var origlength=bytearray_compress.length;
    bytearray_compress.compress();
    // test the compressed bytearray values are all different from the original
    var compressstate=(bytearray_compress[0]==0xef ||
                       bytearray_compress[1]==0xbb ||
                       bytearray_compress[2]==0xbf ||
                       bytearray_compress[3]==100 ||
                       bytearray_compress[4]==97);
    // check the compressed length = 15 (small strings compress larger in zlib)
    var compresslength=bytearray_compress.length;
    bytearray_compress.uncompress();
    // check the uncompress/compress length should equal original length 7
    var restoredlength=bytearray_compress.length;
    var restorestate=(bytearray_compress[0]==0xef &&
                      bytearray_compress[1]==0xbb &&
                      bytearray_compress[2]==0xbf &&
                      bytearray_compress[3]==100 &&
                      bytearray_compress[4]==97 &&
                      bytearray_compress[5]==110 &&
                      bytearray_compress[6]==33
        );
    Assert.expectEq("ByteArray.compress bytearray length is different",
                origlength==compresslength,false);
    Assert.expectEq("ByteArray.compress bytearray contents differ",
                compressstate,false);
    Assert.expectEq("ByteArray.uncompress bytearray length matches before compress",
                origlength,restoredlength);
    Assert.expectEq("ByteArray.uncompress uncompressing compressed string matches original",
                restorestate,true);
}

testBOM();
 
