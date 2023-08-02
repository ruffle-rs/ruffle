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


//     var SECTION = "mops";
//     var VERSION = "AS3";
//     var TITLE   = "sf32";


    Assert.expectError("sf32(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH) prior to initMemory()",
                 Utils.RANGEERROR+1506,
                 function(){ SF32(0x41460200, ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH); });

    initMemory();
    // Get a handle to the domainMemory after it is initialized
    var mem:ByteArray = ApplicationDomain.currentDomain.domainMemory;

    var idx:int=0;
    SI8(0x01, idx+1);
    SI8(0x01, idx+2);

    // Test the memory boundaries
    clearMemory();
    Assert.expectError("sf32(0x41460200, -1)", Utils.RANGEERROR+1506, function(){ SF32(0x41460200, -1); });
    Assert.expectError("sf32(0x41460200, mem.length)", Utils.RANGEERROR+1506, function(){ SF32(0x41460200, mem.length); });
    Assert.expectError("sf32(0x41460200, mem.length-1)", Utils.RANGEERROR+1506, function(){ SF32(0x41460200, mem.length-1); });
    Assert.expectError("sf32(0x41460200, mem.length-2)", Utils.RANGEERROR+1506, function(){ SF32(0x41460200, mem.length-2); });
    Assert.expectError("sf32(0x41460200, mem.length-3)", Utils.RANGEERROR+1506, function(){ SF32(0x41460200, mem.length-3); });

    Assert.expectEq("sf32(0x41460200, mem.length-4)", undefined, SF32(0x01010101, mem.length-4));

    testli8();
    testli16();
    testli32();
    testlf32();
    testreadByte();
    testreadUnsignedByte();
    testreadInt();
    testreadUnsignedInt();
    testreadFloat();


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
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        Assert.expectEq("li8 load 1st byte written by sf32(12.37548828125)", uint(0x00), LI8(0));
        Assert.expectEq("li8 load 2nd byte written by sf32(12.37548828125)", uint(0x02), LI8(1));
        Assert.expectEq("li8 load 3rd byte written by sf32(12.37548828125)", uint(0x46), LI8(2));
        Assert.expectEq("li8 load 4th byte written by sf32(12.37548828125)", uint(0x41), LI8(3));
    }

    function testli16():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        Assert.expectEq("li16 load bytes written by sf32()", 0x0200, LI16(0));
        Assert.expectEq("li16 load bytes written by sf32()", 0x4146, LI16(2));
    }

    function testli32():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        Assert.expectEq("li32 load bytes written by sf32()", int(0x41460200), LI32(0));
    }

    function testlf32():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        Assert.expectEq("lf32 load bytes written by sf32()", 12.37548828125, LF32(0));

    }

    function testreadByte():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        mem.position = 0;
        Assert.expectEq("readByte() load bytes written by sf32()", 0x00, mem.readByte());
        Assert.expectEq("readByte() load bytes written by sf32()", 0x02, mem.readByte());
        Assert.expectEq("readByte() load bytes written by sf32()", 0x46, mem.readByte());
        Assert.expectEq("readByte() load bytes written by sf32()", 0x41, mem.readByte());
    }

    function testreadUnsignedByte():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        mem.position = 0;
        Assert.expectEq("readUnsignedByte() load bytes written by sf32()", 0x00, mem.readUnsignedByte());
        Assert.expectEq("readUnsignedByte() load bytes written by sf32()", 0x02, mem.readUnsignedByte());
        Assert.expectEq("readUnsignedByte() load bytes written by sf32()", 0x46, mem.readUnsignedByte());
        Assert.expectEq("readUnsignedByte() load bytes written by sf32()", 0x41, mem.readUnsignedByte());
    }

    function testreadInt():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        mem.position = 0;
        Assert.expectEq("readInt() load bytes written by sf32()", int(0x41460200), mem.readInt());
    }

    function testreadUnsignedInt():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        mem.position = 0;
        Assert.expectEq("readUnsignedInt() load bytes written by sf32()", uint(0x41460200), mem.readUnsignedInt());
    }

    function testreadFloat():void
    {
        clearMemory();
        // 0x41460200 == 12.37548828125
        SF32(12.37548828125, 0);
        mem.position = 0;
        Assert.expectEq("readFloat() load bytes written by sf32()", 12.37548828125, mem.readFloat());
    }

}
