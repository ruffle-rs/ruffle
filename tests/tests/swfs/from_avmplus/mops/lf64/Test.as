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
//     var TITLE:String   = "lf64";


    Assert.expectError("lf64(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH) prior to initMemory()",
                 Utils.RANGEERROR+1506,
                 function(){ LF64(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH); });

    initMemory();
    // Get a handle to the domainMemory after it is initialized
    var mem:ByteArray = ApplicationDomain.currentDomain.domainMemory;

    Assert.expectError("lf64(-1)", Utils.RANGEERROR+1506, function(){ LF64(-1); });
    Assert.expectError("lf64(mem.length)", Utils.RANGEERROR+1506, function(){ LF64(mem.length); });
    Assert.expectError("lf64(mem.length-1)", Utils.RANGEERROR+1506, function(){ LF64(mem.length-1); });
    Assert.expectError("lf64(mem.length-2)", Utils.RANGEERROR+1506, function(){ LF64(mem.length-2); });
    Assert.expectError("lf64(mem.length-3)", Utils.RANGEERROR+1506, function(){ LF64(mem.length-3); });
    Assert.expectError("lf64(mem.length-4)", Utils.RANGEERROR+1506, function(){ LF64(mem.length-4); });
    Assert.expectError("lf64(mem.length-5)", Utils.RANGEERROR+1506, function(){ LF64(mem.length-5); });
    Assert.expectError("lf64(mem.length-6)", Utils.RANGEERROR+1506, function(){ LF64(mem.length-6); });
    Assert.expectError("lf64(mem.length-7)", Utils.RANGEERROR+1506, function(){ LF64(mem.length-7); });
    Assert.expectEq("lf64(mem.length-8)", 0, LF64(mem.length-8));

    SI32(0x4237D796, 5); // 0x4237D796EFC00000 == 102401241024
    SI32(0xEFC00000, 1); // 0x4237D796EFC00000 == 102401241024
    Assert.expectEq("lf64(1) loads do not need to be aligned", 102401241024, LF64(1));

    testsi8();
    testsi16();
    testsi32();
    testsf32();
    testsf64();
    testwriteByte();
    //testwriteBoolean();
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
         * 0x4237D796EFC00000 = 102401241024
         */
        clearMemory();
        SI8(0x00, 0);
        SI8(0x00, 1);
        SI8(0xC0, 2);
        SI8(0xEF, 3);
        SI8(0x96, 4);
        SI8(0xD7, 5);
        SI8(0x37, 6);
        SI8(0x42, 7);
        Assert.expectEq("lf64 load double written by si8()", 102401241024, LF64(0));
    }

    function testsi16():void
    {
        /**
         * 0x4237D796EFC00000 = 102401241024
         */
        clearMemory();
        SI16(0x0000, 0);
        SI16(0xEFC0, 2);
        SI16(0xD796, 4);
        SI16(0x4237, 6);
        Assert.expectEq("lf64 load double written by si16()", 102401241024, LF64(0));
    }

    function testsi32():void
    {
        /**
         * 0x4237D796EFC00000 = 102401241024
         */
        clearMemory();
        SI32(0xEFC00000, 0);
        SI32(0x4237D796, 4);
        Assert.expectEq("lf64 load double written by si32()", 102401241024, LF64(0));
    }

    function testsf32():void
    {
        // Can't use hex representation here since asc will just treat it
        // as an int|Number and not as a float.
        clearMemory();
        SF32(12.37548828125, 0);
        SF32(12.37548828125, 4);
        Assert.expectEq("lf64 load double written by sf32()", 2884608.5099489688873291015625, LF64(0));
    }

    function testsf64():void
    {
        clearMemory();
        SF64(102401241024, 0);
        Assert.expectEq("lflf load double written by sf64(102401241024)", 102401241024, LF64(0));
    }

    function testwriteByte():void
    {
        /**
         * 0x4237D796EFC00000 = 102401241024
         */
        clearMemory();
        mem.position = 0;
        mem.writeByte(0x00);
        mem.writeByte(0x00);
        mem.writeByte(0xC0);
        mem.writeByte(0xEF);
        mem.writeByte(0x96);
        mem.writeByte(0xD7);
        mem.writeByte(0x37);
        mem.writeByte(0x42);

        Assert.expectEq("lf64 load double written by writeByte()", 102401241024, LF64(0));
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
         * 0x4237D796EFC00000 = 102401241024
         *****************************************/
        clearMemory();
        mem.position = 0;
        mem.writeInt(-272629760);  // 0xEFC00000
        mem.writeInt(1110955926);  // 0x4237D796

        Assert.expectEq("lf64 load double written by writeInt()", 102401241024, LF64(0));
    }

    function testwriteFloat():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeFloat(12.37548828125);
        mem.writeFloat(12.37548828125);
        Assert.expectEq("lf64 load double written by writeFloat(12.37548828125)", 2884608.5099489688873291015625, LF64(0));
    }


    function testwriteDouble():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeDouble(102401241024);
        Assert.expectEq("lf64 load double written by writeDouble(102401241024)", 102401241024, LF64(0));

    }
}
