/* -*- Mode: C++; c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//  Test indirect memory access instructions.

package {

    import flash.utils.ByteArray;
    import flash.utils.Endian;
    import flash.system.ApplicationDomain;
import com.adobe.test.Assert;
import com.adobe.test.Utils;


//     var SECTION:String = "mops";
//     var VERSION:String = "AS3";
//     var TITLE:String   = "li16";


    Assert.expectError("li16(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH) prior to initMemory()",
                 Utils.RANGEERROR+1506,
                 function(){ LI16(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH); });

    initMemory();
    // Get a handle to the domainMemory after it is initialized
    var mem:ByteArray = ApplicationDomain.currentDomain.domainMemory;

    Assert.expectError("li16(-1)", Utils.RANGEERROR+1506, function(){ LI16(-1); });
    Assert.expectError("li16(mem.length)", Utils.RANGEERROR+1506, function(){ LI16(mem.length); });
    Assert.expectError("li16(mem.length-1)", Utils.RANGEERROR+1506, function(){ LI16(mem.length-1); });
    Assert.expectEq("li16(mem.length-2)", 0, LI16(mem.length-2));

    SI16(0x7FDE, 1);
    Assert.expectEq("li16(1) loads do not need to be aligned", 0x7FDE, LI16(1));

    testsi8();
    testsi16();
    testsi32();
    testsf32();
    testsf64();
    testwriteByte();
    testwriteBoolean();
    testwriteInt();
    testwriteFloat();
    testwriteDouble();


    function initMemory(bytes:int = 0):void
    {
        var min:int = ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH;
        var memory:ByteArray = new ByteArray();
        // memory opcodes use native endianness, but ByteArray defaults to BigEndian
        memory.endian = Endian.LITTLE_ENDIAN;
        memory.length = bytes > min ? bytes : min;
        ApplicationDomain.currentDomain.domainMemory = memory;
    }

    function clearMemory():void
    {
        var i:int;
        var len:int = ApplicationDomain.currentDomain.domainMemory.length;
        for ( i=0; i < len; i++)
            SI8(0x00, i);
    }

    function testsi8():void
    {
        clearMemory();
        SI8(0x7F, 0);
        SI8(0x80, 1);
        Assert.expectEq("li16 load short written by si8()", 0x807F, LI16(0));
    }

    function testsi16():void
    {
        clearMemory();
        SI16(0x80DE, 0);
        Assert.expectEq("li16 load short written by si16(0x80DE)", 0x80DE, LI16(0));
    }

    function testsi32():void
    {
        clearMemory();
        SI32(0x80DE32F1, 0);
        Assert.expectEq("li16 load 1st short written by si32(0x80DE32F1)", 0x80DE, LI16(2));
        Assert.expectEq("li16 load 2nd short written by si32(0x80DE32F1)", 0x32F1, LI16(0));
    }

    function testsf32():void
    {
        /******************************************
         * 12.375f
         * (12.375)10 =
         * (12)10 + (0.375)10 =
         * (1100)2 + (0.011)2 =
         * (1100.011)2 =
         * (1.100011)2 x2^3
         * sign = 0
         * exponent = 130 (127 biased, 127 +3, binary 1000 0010)
         * fraction = 100011
         * 0-10000010-10001100000000000000000
         * 01000001 01000110 00000000 00000000 -> int 1095106560 -> 0x41460000
         *****************************************/
        clearMemory();
        SF32(12.375, 0);
        Assert.expectEq("li16 load 1st short written by sf32(12.375)", 0x4146, LI16(2));
        Assert.expectEq("li16 load 2nd short written by sf32(12.375)", 0x0000, LI16(0));
    }

    function testsf64():void
    {
        /******************************************
         * 10241024102410241024
         * 0x43E1C3EDA52E0C09
         * sign = 0
         * exponent = 10000111110
         * mantissa = 0001110000111110110110100101001011100000110000001001
         * 0x43E1C3EDA52E0C09 =
         * 01000011 11100001 11000011 11101101
         * 10100101 00101110 00001100 00001001
         *****************************************/
        clearMemory();
        SF64(1.0241024102410242048E19, 0);
        Assert.expectEq("li16 load 1st short written by sf64(10241024102410241024)", 0x43E1, LI16(6));
        Assert.expectEq("li16 load 2nd short written by sf64(10241024102410241024)", 0xC3ED, LI16(4));
        Assert.expectEq("li16 load 3rd short written by sf64(10241024102410241024)", 0xA52E, LI16(2));
        Assert.expectEq("li16 load 4th short written by sf64(10241024102410241024)", 0x0C09, LI16(0));
    }

    function testwriteByte():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeByte(127);
        mem.writeByte(128);

        Assert.expectEq("li16 load short written by writeByte()", 0x807F, LI16(0));
    }

    function testwriteBoolean():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeBoolean(false);
        mem.writeBoolean(true);

        Assert.expectEq("li16 load short written by writeBoolean()", 0x0100, LI16(0));
    }

    function testwriteInt():void
    {
        /******************************************
         * 2147473647 = 0x7FFFD8EF = 01111111 11111111 11011000 11101111
         * 01111111 = 127 = 0x7F
         * 11111111 = 255 = 0xFF
         * 11011000 = 216 = 0xD8
         * 11101111 = 239 = 0xEF
         *****************************************/
        clearMemory();
        mem.position = 0;
        mem.writeInt(2147473647);

        Assert.expectEq("li16 load 1st short written by writeInt(2147473647)", 0x7FFF, LI16(2));
        Assert.expectEq("li16 load 2nd short written by writeInt(2147473647)", 0xD8EF, LI16(0));
    }

    function testwriteFloat():void
    {
        /******************************************
         * 12.375f
         * (12.375)10 =
         * (12)10 + (0.375)10 =
         * (1100)2 + (0.011)2 =
         * (1100.011)2 =
         * (1.100011)2 x2^3
         * sign = 0
         * exponent = 130 (127 biased, 127 +3, binary 1000 0010)
         * fraction = 100011
         * 0-10000010-10001100000000000000000
         * 01000001 01000110 00000000 00000000 -> int 1095106560 -> 0x41460000
         *****************************************/
        clearMemory();
        mem.position = 0;
        mem.writeFloat(12.375);
        Assert.expectEq("li16 load 1st short written by writeFloat(12.375)", 0x4146, LI16(2));
        Assert.expectEq("li16 load 2nd short written by writeFloat(12.375)", 0x0000, LI16(0));
    }


    function testwriteDouble():void
    {
        /******************************************
         * 10241024102410241024
         * 0x43E1C3EDA52E0C09
         * sign = 0
         * exponent = 10000111110
         * mantissa = 0001110000111110110110100101001011100000110000001001
         * 0x43E1C3EDA52E0C09 =
         * 01000011 11100001 11000011 11101101
         * 10100101 00101110 00001100 00001001
         *****************************************/
        clearMemory();
        mem.position = 0;
        mem.writeDouble(1.0241024102410242048E19);
        Assert.expectEq("li16 load 1st short written by writeDouble(1.0241024102410242048E19)", 0x43E1, LI16(6));
        Assert.expectEq("li16 load 2nd short written by writeDouble(1.0241024102410242048E19)", 0xC3ED, LI16(4));
        Assert.expectEq("li16 load 3rd short written by writeDouble(1.0241024102410242048E19)", 0xA52E, LI16(2));
        Assert.expectEq("li16 load 4th short written by writeDouble(1.0241024102410242048E19)", 0x0C09, LI16(0));

    }

}
