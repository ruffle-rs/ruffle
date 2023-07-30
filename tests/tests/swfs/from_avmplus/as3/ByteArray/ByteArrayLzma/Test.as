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

import flash.errors.EOFError;
import flash.errors.IOError;
import flash.errors.MemoryError;
import flash.utils.ByteArray;
import flash.utils.CompressionAlgorithm;

import com.adobe.test.Assert;

// var SECTION = "ByteArrayWithLzmaTemplate";
// var VERSION = "as3";
// var TITLE   = "test ByteArray class with lzma";


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

function callCompress(byteArray:ByteArray,description:String):void
{
    var error_status:String='no error';
    var len:int = byteArray.length;
    try
    {
        byteArray.compress(CompressionAlgorithm.LZMA);
        byteArray.uncompress(CompressionAlgorithm.LZMA);
    }
    catch(error:Error)
    {
        error_status = error.message;
    }
    byteArray.position = len;
    Assert.expectEq(description+": expecting no exception","no error",error_status);
    Assert.expectEq(description+": expecting length no change ",len,byteArray.length);
}

function testEmptyByteArrayWithLzma()
{
    var bytearray:ByteArray = new ByteArray();
    callCompress(bytearray,"empty bytearray");
}

testEmptyByteArrayWithLzma();

function zeroByteArrayWithLzma()
{
    var bytearray:ByteArray = new ByteArray();
    bytearray.writeByte(0);
    callCompress(bytearray,"zero length bytearray");
}
zeroByteArrayWithLzma();

function nullByteArrayWithLzma()
{
    var byteArray:ByteArray = null;
    var error_status:String='no error';
    try
    {
        byteArray.compress(CompressionAlgorithm.LZMA);
        byteArray.uncompress(CompressionAlgorithm.LZMA);
    }
    catch(error:Error)
    {
        error_status ='error';
    }
    Assert.expectEq("expecting thrown exception","error",error_status);

}

nullByteArrayWithLzma();

function compressWithZlibUncompressLzma()
{
    var byteArray:ByteArray = new ByteArray();
    byteArray.writeUTF("COMPRESS TEST");
    var error_status:String = 'no error';
    try
    {
        byteArray.compress(CompressionAlgorithm.ZLIB)
        byteArray.uncompress(CompressionAlgorithm.LZMA);
    }
    catch(error:Error)
    {
        error_status ='error';
    }

    Assert.expectEq("expecting thrown exception compressWithZlibUncompressLzma", "error", error_status);
}

compressWithZlibUncompressLzma()

function uncompressWithoutCompressionLzma()
{
    var byteArray:ByteArray = new ByteArray();
    byteArray.writeUTF("UNCOMPRESS TEST");

    var uncompressedLengthBefore : uint = byteArray.length;

    var error_status:String = 'no error';

    try
    {
        byteArray.uncompress(CompressionAlgorithm.LZMA);
    }
    catch(error:IOError)
    {
        error_status ='error';
    }
    catch(error:MemoryError)
    {
        error_status ='error';
    }

    Assert.expectEq("expecting thrown exception uncompressWithZlibUncompressLzma", "error", error_status);

    Assert.expectEq("uncompressWithoutCompressionLzma bytearray length matches before uncompress",
        uncompressedLengthBefore, byteArray.length);
}

uncompressWithoutCompressionLzma()

function testBooleanWithLzma()
{
    var bytearray:ByteArray=makeByteArray();
    bytearray.writeBoolean(true);
    bytearray.writeBoolean(false);
    Assert.expectEq("ByteArray position after writing Booleans",
        2,
        bytearray.position);
    callCompress(bytearray,"boolean bytearray");
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

testBooleanWithLzma();

function testShortWithLzma()
{
    // One endianness or the other
    function readShort_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeShort(100);
        bytearray.writeShort(-200);
        callCompress(bytearray,"bytearray of short");
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
        callCompress(bytearray,"bytearray of short mixed endianness");
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
        callCompress(bytearray,"bytearray of shorts with EOFs at various offsets");

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

testShortWithLzma();

function testUnsignedShortWithLzma()
{
    // One endianness or the other
    function readUShort_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeShort(100);
        bytearray.writeShort(uint(-200) & 65535);
        callCompress(bytearray,"unsigned short");
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
        callCompress(bytearray,"unsigned short mixed endianness");
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
        callCompress(bytearray,"unsigned short eof at various offsets");
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

testUnsignedShortWithLzma();

function testIntWithLzma()
{
    // One endianness or the other
    function readInt_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeInt(100);
        bytearray.writeInt(-200);
        callCompress(bytearray,"int endianness");
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
        callCompress(bytearray,"int mixed endianness");
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
    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readInt_tests1("bigEndian", offs);
        readInt_tests1("littleEndian", offs);
        readInt_tests2(offs);
    }
}

testIntWithLzma();

function testFloatWithLzma()
{
    // One endianness or the other
    function readFloat_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeFloat(1.25);
        bytearray.writeFloat(12345.5);
        callCompress(bytearray,"float");
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
        var temp:ByteArray=new ByteArray();
        
        bytearray.endian = "bigEndian";
        bytearray.position=offset;
        
        bytearray.writeFloat(1.25);    // write big
        bytearray.writeFloat(12345.5); //   endian
        callCompress(bytearray,"float mixed endianness");
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
    
    
    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readFloat_tests1("bigEndian", offs);
        readFloat_tests1("littleEndian", offs);
        readFloat_tests2(offs);
    }
}

testFloatWithLzma();

function testDoubleWithLzma() 
{
    // One endianness or the other
    function readDouble_tests1(endian, offset)
    {
        var bytearray:ByteArray=makeByteArray(offset);
        bytearray.endian = endian;
        bytearray.position=offset;
        bytearray.writeDouble(1.25);
        bytearray.writeDouble(12345.5);
        callCompress(bytearray,"double");
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
        callCompress(bytearray,"double mixed endianness");
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
    
    for ( var offs=0 ; offs < 4 ; offs++ ) {
        readDouble_tests1("bigEndian", offs);
        readDouble_tests1("littleEndian", offs);
        readDouble_tests2(offs);
        
    }
}

testDoubleWithLzma();

function testByteWithLzma() 
{
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeByte(-257);
    bytearray.writeByte(37);
    callCompress(bytearray,"byte");
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

testByteWithLzma();

function testUtfWithLzma() 
{
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeUTF("string");
    callCompress(bytearray,"UTF");
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
    callCompress(bytearray,"bytes");
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
    callCompress(bytearray,"skip UTF-8 BOM");
    bytearray.position = 0;
    Assert.expectEq("ByteArray readUTF skips UTF8 BOM after length bytes but includes it in the length",
        "str",
        bytearray.readUTF());
    
    // TODO: test invalid UTF - we should still get data, in a predictable way (invalid input turns into individual bytes)
}

testUtfWithLzma();

function testUtfBytesWithLzma() 
{
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeUTFBytes("string");
    callCompress(bytearray,"UTF bytes");
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
    callCompress(bytearray,"bytes contains NULL");
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
    callCompress(bytearray,"bytes contain high value bytes");
    bytearray.position = 0;
    Assert.expectEq("ByteArray readUTFBytes skips UTF8 BOM but includes it in the length",
        "str",
        bytearray.readUTFBytes(6));
    
    // TODO: test invalid UTF - we should still get data, in a predictable way (invalid input turns into individual bytes)
}

testUtfBytesWithLzma();


function testBracketSyntaxWithLzma() {
    var bytearray:ByteArray = makeByteArray();
    bytearray.position=0;
    bytearray.writeByte(10);
    bytearray.writeByte(11);
    bytearray.writeByte(12);
    callCompress(bytearray,"bracket syntax");
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
    
}

testBracketSyntaxWithLzma(); 


function testBOMWithLzma() {   
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
    bytearray_compress.compress("lzma");
    // test the compressed bytearray values are all different from the original
    var compressstate=(bytearray_compress[0]==0xef ||
        bytearray_compress[1]==0xbb ||
        bytearray_compress[2]==0xbf ||
        bytearray_compress[3]==100 ||
        bytearray_compress[4]==97);
    // check the compressed length = 15 (small strings compress larger in zlib)
    var compresslength=bytearray_compress.length;
    bytearray_compress.uncompress("lzma");
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

testBOMWithLzma();

    // lzma-compressed data format:
    // 5 bytes: LZMA properties
    // 8 bytes: uncompressed size k (little-endian)
    // k bytes: payload (the compressed data)

function testIllFormedLzmaLowSize():void {
    var b1:ByteArray = new ByteArray();
    b1.writeUTF("string");
    b1.compress(CompressionAlgorithm.LZMA);

    var b2:ByteArray = new ByteArray();

    // muck with low 32 bits of size
    for (var i:uint=0; i < 4; i++) {
        b2.clear();
        b1.position = 0;
        b1.readBytes(b2);

        b2[5 + i] += 1;

        var result;
        try {
            b2.uncompress(CompressionAlgorithm.LZMA);
            result = "unexpected"
        } catch (e:IOError) {
            result = "ioerror on wrong size"
        }

        Assert.expectEq("ByteArray.uncompress ill-formed LZMA size "+i+" byte",
                    "ioerror on wrong size",result);
    }
}
// comment out test due to bug: https://bugzilla.mozilla.org/show_bug.cgi?id=778727
//testIllFormedLzmaLowSize();

function testIllFormedLzmaHighSize():void {
    var b1:ByteArray = new ByteArray();
    b1.writeUTF("string");
    b1.compress(CompressionAlgorithm.LZMA);

    var b2:ByteArray = new ByteArray();


    // muck with high 32 bits of size
    for (var i:uint=4; i < 8; i++) {
        b2.clear();
        b1.position = 0;
        b1.readBytes(b2);

        b2[5 + i] += 1;

        var result;
        try {
            b2.uncompress(CompressionAlgorithm.LZMA);
            result = "unexpected"
        } catch (e:MemoryError) {
            result = "memoryerror on huge size"
        }

        Assert.expectEq("ByteArray.uncompress ill-formed LZMA size "+i+" byte",
                    "memoryerror on huge size",result);
    }
}
// comment out test due to bug: https://bugzilla.mozilla.org/show_bug.cgi?id=778727
// testIllFormedLzmaHighSize();

// (Below is not necessarily working; in particular Felix would have expected
//  it to hit the IllFormedLzma cases above, but for some reason it is not.)
function testFuzzedLzma():void {
    var b1:ByteArray = new ByteArray();
    b1.writeUTF("string");
    b1.compress(CompressionAlgorithm.LZMA);

    var b2:ByteArray = new ByteArray();

    var errors:String="";
    for (var i:uint=0; i < b1.length * 8; i++) {
        b2.clear();
        b1.position = 0;
        //        print("b1.length A: "+b1.length);
        b1.readBytes(b2);
        //        print("b1.length B: "+b1.length);

        var byteOffset:uint = i / 8;
        var bitOffset:uint  = i % 8;

        var oldByte:uint = b2[byteOffset];
        var oldBit:uint  = (oldByte >> bitOffset) & 0x1;
        var newByte:uint = (oldByte ^ (oldBit << bitOffset)) & 0xFF;
        b2[byteOffset] = newByte;

        //        print('b1 ['+Array.prototype.join.call(b1, ',')+']');
        //        print('b2 ['+Array.prototype.join.call(b2, ',')+']');

        var result;
        // This test is largely fishing for segfaults (and striving for code coverage)
        try {
            b2.uncompress(CompressionAlgorithm.LZMA);

            // If we get here, then the LZMA algorithm completed and
            // we have no idea what the resulting byte array contains.
            result = "expected" // no exn
        } catch (e:MemoryError) {
            result = "expected"
        } catch (e:IOError) {
            // should also verify that original data was restored
            result = "expected"
        }
        if (result!="expected") {
            errors+=" error fuzzing bit "+i+" ";
        }
    }
    Assert.expectEq("ByteArray.uncompress fuzzed LZMA input check for errors",
                    "",errors);
}

// Commenting out test because it is not behaving the way Felix would
// expect and he does not want to take the time now to debug it.

// comment out test due to bug: https://bugzilla.mozilla.org/show_bug.cgi?id=778727
// testFuzzedLzma();

