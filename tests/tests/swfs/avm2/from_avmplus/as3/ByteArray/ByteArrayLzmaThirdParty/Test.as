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
import flash.utils.ByteArray;
import flash.utils.ByteArray;
import flash.utils.ByteArray;
import flash.utils.CompressionAlgorithm;

import com.adobe.test.Assert;

// var SECTION = "ByteArrayWithLzmaThirdParty";
// var VERSION = "as3";
// var TITLE   = "test ByteArray class with lzma inputs generated via LZMA.jar";

var abc_compressed_hello = new ByteArray();
abc_compressed_hello.writeByte(0x5D);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x80);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x92);
abc_compressed_hello.writeByte(0x01);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x24);
abc_compressed_hello.writeByte(0x19);
abc_compressed_hello.writeByte(0x49);
abc_compressed_hello.writeByte(0x98);
abc_compressed_hello.writeByte(0x6F);
abc_compressed_hello.writeByte(0x10);
abc_compressed_hello.writeByte(0x11);
abc_compressed_hello.writeByte(0xC8);
abc_compressed_hello.writeByte(0x5F);
abc_compressed_hello.writeByte(0xE6);
abc_compressed_hello.writeByte(0xD5);
abc_compressed_hello.writeByte(0x8A);
abc_compressed_hello.writeByte(0x64);
abc_compressed_hello.writeByte(0x72);
abc_compressed_hello.writeByte(0x70);
abc_compressed_hello.writeByte(0x9E);
abc_compressed_hello.writeByte(0xA8);
abc_compressed_hello.writeByte(0x41);
abc_compressed_hello.writeByte(0x82);
abc_compressed_hello.writeByte(0x52);
abc_compressed_hello.writeByte(0x13);
abc_compressed_hello.writeByte(0x1B);
abc_compressed_hello.writeByte(0x09);
abc_compressed_hello.writeByte(0xB2);
abc_compressed_hello.writeByte(0x30);
abc_compressed_hello.writeByte(0x51);
abc_compressed_hello.writeByte(0xAD);
abc_compressed_hello.writeByte(0x62);
abc_compressed_hello.writeByte(0x82);
abc_compressed_hello.writeByte(0xA4);
abc_compressed_hello.writeByte(0x1B);
abc_compressed_hello.writeByte(0x14);
abc_compressed_hello.writeByte(0x99);
abc_compressed_hello.writeByte(0xF4);
abc_compressed_hello.writeByte(0xBB);
abc_compressed_hello.writeByte(0xCB);
abc_compressed_hello.writeByte(0x46);
abc_compressed_hello.writeByte(0xF9);
abc_compressed_hello.writeByte(0x2F);
abc_compressed_hello.writeByte(0x5D);
abc_compressed_hello.writeByte(0x05);
abc_compressed_hello.writeByte(0x6F);
abc_compressed_hello.writeByte(0xA1);
abc_compressed_hello.writeByte(0xA0);
abc_compressed_hello.writeByte(0x46);
abc_compressed_hello.writeByte(0xB7);
abc_compressed_hello.writeByte(0x9D);
abc_compressed_hello.writeByte(0x4C);
abc_compressed_hello.writeByte(0x1A);
abc_compressed_hello.writeByte(0x7F);
abc_compressed_hello.writeByte(0xB4);
abc_compressed_hello.writeByte(0xD4);
abc_compressed_hello.writeByte(0xFC);
abc_compressed_hello.writeByte(0x7C);
abc_compressed_hello.writeByte(0x4B);
abc_compressed_hello.writeByte(0x83);
abc_compressed_hello.writeByte(0x84);
abc_compressed_hello.writeByte(0x97);
abc_compressed_hello.writeByte(0x7C);
abc_compressed_hello.writeByte(0x25);
abc_compressed_hello.writeByte(0xCB);
abc_compressed_hello.writeByte(0x0E);
abc_compressed_hello.writeByte(0xA4);
abc_compressed_hello.writeByte(0xEB);
abc_compressed_hello.writeByte(0x5C);
abc_compressed_hello.writeByte(0xD5);
abc_compressed_hello.writeByte(0x69);
abc_compressed_hello.writeByte(0x91);
abc_compressed_hello.writeByte(0xE0);
abc_compressed_hello.writeByte(0xE3);
abc_compressed_hello.writeByte(0x1B);
abc_compressed_hello.writeByte(0xD9);
abc_compressed_hello.writeByte(0x8A);
abc_compressed_hello.writeByte(0x7F);
abc_compressed_hello.writeByte(0x63);
abc_compressed_hello.writeByte(0x44);
abc_compressed_hello.writeByte(0xB7);
abc_compressed_hello.writeByte(0x89);
abc_compressed_hello.writeByte(0x36);
abc_compressed_hello.writeByte(0x82);
abc_compressed_hello.writeByte(0x68);
abc_compressed_hello.writeByte(0x6F);
abc_compressed_hello.writeByte(0xBD);
abc_compressed_hello.writeByte(0x1C);
abc_compressed_hello.writeByte(0x3F);
abc_compressed_hello.writeByte(0x1F);
abc_compressed_hello.writeByte(0xE5);
abc_compressed_hello.writeByte(0xC1);
abc_compressed_hello.writeByte(0xF9);
abc_compressed_hello.writeByte(0xE5);
abc_compressed_hello.writeByte(0x36);
abc_compressed_hello.writeByte(0xB4);
abc_compressed_hello.writeByte(0x08);
abc_compressed_hello.writeByte(0x71);
abc_compressed_hello.writeByte(0x14);
abc_compressed_hello.writeByte(0xAC);
abc_compressed_hello.writeByte(0x9E);
abc_compressed_hello.writeByte(0xEC);
abc_compressed_hello.writeByte(0x24);
abc_compressed_hello.writeByte(0x82);
abc_compressed_hello.writeByte(0x77);
abc_compressed_hello.writeByte(0x5E);
abc_compressed_hello.writeByte(0x68);
abc_compressed_hello.writeByte(0x00);
abc_compressed_hello.writeByte(0x23);
abc_compressed_hello.writeByte(0x75);
abc_compressed_hello.writeByte(0x68);
abc_compressed_hello.writeByte(0xEE);
abc_compressed_hello.writeByte(0x03);
abc_compressed_hello.writeByte(0x9A);
abc_compressed_hello.writeByte(0x62);
abc_compressed_hello.writeByte(0x2D);
abc_compressed_hello.writeByte(0xFE);
abc_compressed_hello.writeByte(0xA0);
abc_compressed_hello.writeByte(0x72);
abc_compressed_hello.writeByte(0x13);
abc_compressed_hello.writeByte(0x80);
abc_compressed_hello.writeByte(0x58);
abc_compressed_hello.writeByte(0x8B);
abc_compressed_hello.writeByte(0x79);
abc_compressed_hello.writeByte(0x63);
abc_compressed_hello.writeByte(0x6E);
abc_compressed_hello.writeByte(0x14);
abc_compressed_hello.writeByte(0xF3);
abc_compressed_hello.writeByte(0x72);
abc_compressed_hello.writeByte(0x70);
abc_compressed_hello.writeByte(0x4F);
abc_compressed_hello.writeByte(0xFD);
abc_compressed_hello.writeByte(0x81);
abc_compressed_hello.writeByte(0xCA);
abc_compressed_hello.writeByte(0x3D);
abc_compressed_hello.writeByte(0xD5);
abc_compressed_hello.writeByte(0xB6);
abc_compressed_hello.writeByte(0x6F);
abc_compressed_hello.writeByte(0xD2);
abc_compressed_hello.writeByte(0xAF);
abc_compressed_hello.writeByte(0x79);
abc_compressed_hello.writeByte(0x09);
abc_compressed_hello.writeByte(0xE5);
abc_compressed_hello.writeByte(0x27);
abc_compressed_hello.writeByte(0x03);
abc_compressed_hello.writeByte(0x8C);
abc_compressed_hello.writeByte(0x2F);
abc_compressed_hello.writeByte(0x73);
abc_compressed_hello.writeByte(0x29);
abc_compressed_hello.writeByte(0xED);
abc_compressed_hello.writeByte(0xAC);
abc_compressed_hello.writeByte(0xC2);
abc_compressed_hello.writeByte(0xD9);
abc_compressed_hello.writeByte(0xC3);
abc_compressed_hello.writeByte(0x86);
abc_compressed_hello.writeByte(0x27);
abc_compressed_hello.writeByte(0x38);
abc_compressed_hello.writeByte(0x23);
abc_compressed_hello.writeByte(0xDC);
abc_compressed_hello.writeByte(0x84);
abc_compressed_hello.writeByte(0x52);
abc_compressed_hello.writeByte(0xA7);
abc_compressed_hello.writeByte(0x9F);
abc_compressed_hello.writeByte(0xF7);
abc_compressed_hello.writeByte(0x5F);
abc_compressed_hello.writeByte(0xF3);
abc_compressed_hello.writeByte(0x1B);
abc_compressed_hello.writeByte(0x7E);
abc_compressed_hello.writeByte(0x74);
abc_compressed_hello.writeByte(0x57);
abc_compressed_hello.writeByte(0xD6);
abc_compressed_hello.writeByte(0xEB);
abc_compressed_hello.writeByte(0x62);
abc_compressed_hello.writeByte(0xF4);
abc_compressed_hello.writeByte(0x31);
abc_compressed_hello.writeByte(0x3B);
abc_compressed_hello.writeByte(0x11);
abc_compressed_hello.writeByte(0xE5);
abc_compressed_hello.writeByte(0x50);
abc_compressed_hello.writeByte(0x1C);
abc_compressed_hello.writeByte(0x49);
abc_compressed_hello.writeByte(0x10);
abc_compressed_hello.writeByte(0x61);
abc_compressed_hello.writeByte(0xC9);
abc_compressed_hello.writeByte(0x5D);
abc_compressed_hello.writeByte(0x1C);
abc_compressed_hello.writeByte(0x15);
abc_compressed_hello.writeByte(0x45);
abc_compressed_hello.writeByte(0x87);
abc_compressed_hello.writeByte(0x55);
abc_compressed_hello.writeByte(0x10);
abc_compressed_hello.writeByte(0x21);
abc_compressed_hello.writeByte(0x7F);
abc_compressed_hello.writeByte(0x83);
abc_compressed_hello.writeByte(0x1B);
abc_compressed_hello.writeByte(0xFD);
abc_compressed_hello.writeByte(0x8E);
abc_compressed_hello.writeByte(0x4C);
abc_compressed_hello.writeByte(0xD1);
abc_compressed_hello.writeByte(0x9B);
abc_compressed_hello.writeByte(0x27);
abc_compressed_hello.writeByte(0x01);
abc_compressed_hello.writeByte(0x0E);
abc_compressed_hello.writeByte(0x35);
abc_compressed_hello.writeByte(0x34);
abc_compressed_hello.writeByte(0xFB);
abc_compressed_hello.writeByte(0x1D);
abc_compressed_hello.writeByte(0xA7);
abc_compressed_hello.writeByte(0xA1);
abc_compressed_hello.writeByte(0xA9);
abc_compressed_hello.writeByte(0x1A);
abc_compressed_hello.writeByte(0x42);
abc_compressed_hello.writeByte(0xAB);
abc_compressed_hello.writeByte(0x4F);
abc_compressed_hello.writeByte(0xA3);
abc_compressed_hello.writeByte(0x82);
abc_compressed_hello.writeByte(0xA7);
abc_compressed_hello.writeByte(0x37);
abc_compressed_hello.writeByte(0x04);
abc_compressed_hello.writeByte(0x95);
abc_compressed_hello.writeByte(0x1E);
abc_compressed_hello.writeByte(0xF8);
abc_compressed_hello.writeByte(0x8E);
abc_compressed_hello.writeByte(0xA5);
abc_compressed_hello.writeByte(0x0F);
abc_compressed_hello.writeByte(0x9A);
abc_compressed_hello.writeByte(0xE4);
abc_compressed_hello.position = 0;

var abc_compressed_small = new ByteArray();
abc_compressed_small.writeByte(0x5D);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x80);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x1C);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x22);
abc_compressed_small.writeByte(0x19);
abc_compressed_small.writeByte(0x49);
abc_compressed_small.writeByte(0x86);
abc_compressed_small.writeByte(0xB0);
abc_compressed_small.writeByte(0x70);
abc_compressed_small.writeByte(0x8E);
abc_compressed_small.writeByte(0xD1);
abc_compressed_small.writeByte(0xE2);
abc_compressed_small.writeByte(0xA2);
abc_compressed_small.writeByte(0x80);
abc_compressed_small.writeByte(0x2D);
abc_compressed_small.writeByte(0xE1);
abc_compressed_small.writeByte(0x85);
abc_compressed_small.writeByte(0x6F);
abc_compressed_small.writeByte(0x1E);
abc_compressed_small.writeByte(0xE6);
abc_compressed_small.writeByte(0xD5);
abc_compressed_small.writeByte(0x4B);
abc_compressed_small.writeByte(0x2A);
abc_compressed_small.writeByte(0x79);
abc_compressed_small.writeByte(0x6C);
abc_compressed_small.writeByte(0x55);
abc_compressed_small.writeByte(0x20);
abc_compressed_small.writeByte(0x8E);
abc_compressed_small.writeByte(0x60);
abc_compressed_small.writeByte(0x9D);
abc_compressed_small.writeByte(0x9A);
abc_compressed_small.writeByte(0x61);
abc_compressed_small.writeByte(0x26);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.writeByte(0x00);
abc_compressed_small.position = 0;


// Bugzilla 733051: ByteArrayLzma callCompress tests are doing
// round-trips through compress and uncompress; this test is checking
// that we correctly handle inputs that have been compressed by
// third-party utilities (see LZMA.jar attached to Bugzilla 729336).
function testThirdPartyLzmaInputs()
{
    var compressedHello:ByteArray = new ByteArray();
    compressedHello.writeBytes(abc_compressed_hello);
    var compressedSmall:ByteArray = new ByteArray();
    compressedSmall.writeBytes(abc_compressed_small);

    var helloString:String = "Hello World!\n" +
      "\n" +
      "More text; Felix wants to illustrate content that is compressed by\n" +
      "LZMA, but a 13 byte file with just \"Hello World!\\n\" is compressed to a\n" +
      "31 byte file by LZMA.  (Though perhaps a large portion is LZMA header?\n" +
      "Not sure yet.)  Easy ending: LZMA LZMA LZMA Hello World LZMA LZMA LZMA\n" +
      "LZMA LZMA LZMA Hello World LZMA LZMA LZMA Malkovich Malkovich LZMA\n" +
      "LZMA LZMA Malkovich LZMA LZMA LZMA LZMA.\n";
    var smallString:String = "Deliberately small snippet.\n";

    compressedHello.uncompress(CompressionAlgorithm.LZMA);
    var helloString2:String = compressedHello.readUTFBytes(compressedHello.length);
    Assert.expectEq("Correct lzma uncompression on hello", helloString, helloString2);

    compressedSmall.uncompress(CompressionAlgorithm.LZMA);
    var smallString2:String = compressedSmall.readUTFBytes(compressedSmall.length);
    Assert.expectEq("Correct lzma uncompression on small", smallString, smallString2);
}

testThirdPartyLzmaInputs();

