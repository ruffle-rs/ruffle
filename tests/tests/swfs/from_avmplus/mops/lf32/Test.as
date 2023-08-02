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
//     var TITLE:String   = "lf32";


    Assert.expectError("lf32(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH) prior to initMemory()",
                 Utils.RANGEERROR+1506,
                 function(){ LF32(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH); });

    initMemory();
    // Get a handle to the domainMemory after it is initialized
    var mem:ByteArray = ApplicationDomain.currentDomain.domainMemory;

    Assert.expectError("lf32(-1)", Utils.RANGEERROR+1506, function(){ LF32(-1); });
    Assert.expectError("lf32(mem.length)", Utils.RANGEERROR+1506, function(){ LF32(mem.length); });
    Assert.expectError("lf32(mem.length-1)", Utils.RANGEERROR+1506, function(){ LF32(mem.length-1); });
    Assert.expectError("lf32(mem.length-2)", Utils.RANGEERROR+1506, function(){ LF32(mem.length-2); });
    Assert.expectError("lf32(mem.length-3)", Utils.RANGEERROR+1506, function(){ LF32(mem.length-3); });
    Assert.expectEq("lf32(mem.length-4)", 0, LF32(mem.length-4));

    SI32(0x41460200, 1); // 0x41460200 == 12.37548828125
    Assert.expectEq("lf32(1) loads do not need to be aligned", 12.37548828125, LF32(1));

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
        /**
         * 0x41460200 = 01000001 01000110 00000010 00000000
         * 12.37548828125
         */
        clearMemory();
        SI8(0x00, 0);
        SI8(0x02, 1);
        SI8(0x46, 2);
        SI8(0x41, 3);
        Assert.expectEq("lf32 load float written by si8()", 12.37548828125, LF32(0));
    }

    function testsi16():void
    {
        /**
         * 0x41460200 = 01000001 01000110 00000010 00000000
         * 12.37548828125
         */
        clearMemory();
        SI16(0x0200, 0);
        SI16(0x4146, 2);
        Assert.expectEq("lf32 load float written by si16()", 12.37548828125, LF32(0));
    }

    function testsi32():void
    {
        clearMemory();
        SI32(0x41460200, 0);
        Assert.expectEq("lf32 load float written by si32(0x41460200)", 12.37548828125, LF32(0));
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
        // Can't use hex representation here since asc will just treat it
        // as an int|Number and not as a float.
        clearMemory();
        SF32(12.37548828125, 0);
        Assert.expectEq("lf32 load float written by sf32(12.37548828125)", 12.37548828125, LF32(0));
    }

    function testsf64():void
    {
        /******************************************
         * 2.8846085099489688873291015625E6
         * 0x4146020041460200
         * sign = 0
         * exponent = 10000010100
         * mantissa = 0110000000100000000001000001010001100000001000000000
         *****************************************/
        clearMemory();
        SF64(2.8846085099489688873291015625E6, 0);
        Assert.expectEq("lf32 load 1st float written by sf64(0x4146020041460200)", 12.37548828125, LF32(4));
        Assert.expectEq("lf32 load 2nd float written by sf64(0x4146020041460200)", 12.37548828125, LF32(0));
    }

    function testwriteByte():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeByte(0x00);
        mem.writeByte(0x02);
        mem.writeByte(0x46);
        mem.writeByte(0x41);

        Assert.expectEq("lf32 load float written by writeByte()", 12.37548828125, LF32(0));
    }

    function testwriteBoolean():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeBoolean(true);
        mem.writeBoolean(false);
        mem.writeBoolean(false);
        mem.writeBoolean(true);

        Assert.expectEq("lf32 load float written by writeBoolean()", 2.3510604481259484465715043694E-38, LF32(0));
    }

    function testwriteInt():void
    {
        /******************************************
         * 1095107072 = 0x41460200
         *****************************************/
        clearMemory();
        mem.position = 0;
        mem.writeInt(1095107072);

        Assert.expectEq("lf32 load float written by writeInt(2147473647)", 12.37548828125, LF32(0));
    }

    function testwriteFloat():void
    {
        /******************************************
         * 12.375f
         *****************************************/
        clearMemory();
        mem.position = 0;
        mem.writeFloat(12.375);
        Assert.expectEq("lf32 load float written by writeFloat(12.375)", 12.375, LF32(0));
    }


    function testwriteDouble():void
    {
        /******************************************
         * 2.8846085099489688873291015625E6
         * 0x4146020041460200
         * sign = 0
         * exponent = 10000010100
         * mantissa = 0110000000100000000001000001010001100000001000000000
         *****************************************/
        clearMemory();
        mem.position = 0;
        mem.writeDouble(2.8846085099489688873291015625E6);
        Assert.expectEq("lf32 load 1st float written by writeDouble(2.8846085099489688873291015625E6)", 12.37548828125, LF32(4));
        Assert.expectEq("lf32 load 2nd float written by writeDouble(2.8846085099489688873291015625E6)", 12.37548828125, LF32(0));

    }

}
