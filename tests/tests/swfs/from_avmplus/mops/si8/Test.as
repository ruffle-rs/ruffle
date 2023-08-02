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
//     var TITLE:String   = "si8";


    Assert.expectError("si8(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH) prior to initMemory()",
                 Utils.RANGEERROR+1506,
                 function(){ SI8(0x01, ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH); });

    initMemory();
    // Get a handle to the domainMemory after it is initialized
    var mem:ByteArray = ApplicationDomain.currentDomain.domainMemory;

    /**
     * Test that ONLY the lowest 8 bits are stored
     */
    clearMemory();
    SI8(0x7F5A, 1); // should only store 0x5A
    SI8(0x62, 0);
    // li16(0) should load 0x5062
    Assert.expectEq("si8() only stores the least significant 8 bits", 0x5A62, LI16(0));
    // li8(2) should load 0x00, 0x7F should not have spilled
    Assert.expectEq("si8() only stores the least significant 8 bits", 0x00, LI8(2));

    // Test the memory boundaries
    clearMemory();
    Assert.expectError("si8(0x01, -1)", Utils.RANGEERROR+1506, function(){ SI8(0x01, -1); });
    Assert.expectError("si8(0x01, mem.length)", Utils.RANGEERROR+1506, function(){ SI8(0x01, mem.length); });
    Assert.expectEq("si8(0x01, mem.length-1)", undefined, SI8(0x01, mem.length-1));
    Assert.expectEq("si8(0x01FE, mem.length-1), should only store 8 bits so no overrun", undefined, SI8(0x01FE, mem.length-1));
    Assert.expectEq("si8(0x01FE, mem.length-1), should only store 8 bits so no overrun confirm", 0xFE, LI8(mem.length-1));


    testli8();
    testli16();
    testli32();
    testlf32();
    testlf64();
    testreadByte();
    testreadUnsignedByte();
    testreadBoolean();
    testreadInt();
    testreadUnsignedInt();
    testreadFloat();
    testreadDouble();


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

    function testli8():void
    {
        clearMemory();
        SI8(0x7F, 0);
        SI8(0x80, 1);
        Assert.expectEq("li8 load byte written by si8(0x7F)", 0x7F, LI8(0));
        Assert.expectEq("li8 load byte written by si8(0x80)", 0x80, LI8(1));
    }

    function testli16():void
    {
        clearMemory();
        SI8(0xDE, 0);
        SI8(0x80, 1);
        Assert.expectEq("li16 load bytes written by si8()", 0x80DE, LI16(0));
    }

    function testli32():void
    {
        clearMemory();
        SI8(0x80, 3);
        SI8(0xDE, 2);
        SI8(0x32, 1);
        SI8(0xF1, 0);
        Assert.expectEq("li32 load bytes written by si8()", int(0x80DE32F1), LI32(0));
    }

    function testlf32():void
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
        SI8(0x41, 3);
        SI8(0x46, 2);
        SI8(0x00, 1);
        SI8(0x00, 0);
        Assert.expectEq("lf32 load bytes written by si8()", 12.375, LF32(0));

    }

    function testlf64():void
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
        SI8(0x43, 7);
        SI8(0xE1, 6);
        SI8(0xC3, 5);
        SI8(0xED, 4);
        SI8(0xA5, 3);
        SI8(0x2E, 2);
        SI8(0x0C, 1);
        SI8(0x09, 0);
        Assert.expectEq("lf64 load bytes written by si8()", 1.0241024102410242048E19, LF64(0));
    }

    function testreadByte():void
    {
        clearMemory();
        SI8(0x00, 0);
        SI8(0x7F, 1);
        SI8(0x80, 2);
        SI8(0xFF, 3);

        mem.position = 0;
        Assert.expectEq("readByte() load bytes written by si8(0x00)", 0, mem.readByte());
        Assert.expectEq("readByte() load bytes written by si8(0x7F)", 127, mem.readByte());
        Assert.expectEq("readByte() load bytes written by si8(0x80)", -128, mem.readByte());
        Assert.expectEq("readByte() load bytes written by si8(0xFF)", -1, mem.readByte());
    }

    function testreadUnsignedByte():void
    {
        clearMemory();
        SI8(0x00, 0);
        SI8(0x7F, 1);
        SI8(0x80, 2);
        SI8(0xFF, 3);

        mem.position = 0;
        Assert.expectEq("readUnsignedByte() load bytes written by si8(0x00)", 0, mem.readUnsignedByte());
        Assert.expectEq("readUnsignedByte() load bytes written by si8(0x7F)", 127, mem.readUnsignedByte());
        Assert.expectEq("readUnsignedByte() load bytes written by si8(0x80)", 128, mem.readUnsignedByte());
        Assert.expectEq("readUnsignedByte() load bytes written by si8(0xFF)", 255, mem.readUnsignedByte());
    }


    function testreadBoolean():void
    {
        clearMemory();
        SI8(0x00, 0);
        SI8(0x01, 1);

        mem.position = 0;
        Assert.expectEq("readBoolean() load bytes written by si8(0x00)", false, mem.readBoolean());
        Assert.expectEq("readBoolean() load bytes written by si8(0x01)", true, mem.readBoolean());
    }

    function testreadInt():void
    {
        clearMemory();
        SI8(0x80, 3);
        SI8(0xDE, 2);
        SI8(0x32, 1);
        SI8(0xF1, 0);

        mem.position = 0;
        Assert.expectEq("readInt() load bytes written by si8()", int(0x80DE32F1), mem.readInt());
    }

    function testreadUnsignedInt():void
    {
        clearMemory();
        SI8(0x80, 3);
        SI8(0xDE, 2);
        SI8(0x32, 1);
        SI8(0xF1, 0);

        mem.position = 0;
        Assert.expectEq("readUnsignedInt() load bytes written by si8()", uint(0x80DE32F1), mem.readUnsignedInt());
    }

    function testreadFloat():void
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
        SI8(0x41, 3);
        SI8(0x46, 2);
        SI8(0x00, 1);
        SI8(0x00, 0);

        mem.position = 0;
        Assert.expectEq("readFloat() load bytes written by si8()", 12.375, mem.readFloat());
    }

    function testreadDouble():void
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
        SI8(0x43, 7);
        SI8(0xE1, 6);
        SI8(0xC3, 5);
        SI8(0xED, 4);
        SI8(0xA5, 3);
        SI8(0x2E, 2);
        SI8(0x0C, 1);
        SI8(0x09, 0);

        mem.position = 0;
        Assert.expectEq("readDouble() load bytes written by si8()", 1.0241024102410242048E19, mem.readDouble());
    }


}
