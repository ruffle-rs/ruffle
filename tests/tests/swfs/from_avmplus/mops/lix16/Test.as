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
//     var TITLE:String   = "lix16";


    Assert.expectError("lix16(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH) prior to initMemory()",
                 Utils.RANGEERROR+1506,
                 function(){ LIX16(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH); });

    initMemory();
    // Get a handle to the domainMemory after it is initialized
    var mem:ByteArray = ApplicationDomain.currentDomain.domainMemory;

    Assert.expectError("lix16(-1)", Utils.RANGEERROR+1506, function(){ LIX16(-1); });
    Assert.expectError("lix16(mem.length)", Utils.RANGEERROR+1506, function(){ LIX16(mem.length); });
    Assert.expectError("lix16(mem.length-1)", Utils.RANGEERROR+1506, function(){ LIX16(mem.length-1); });
    Assert.expectEq("lix16(mem.length-2)", 0, LIX16(mem.length-2));

    testsi8();
    testsi16();
    testsi32();
    testwriteByte();
    testwriteInt();


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
        // 0x8000    -32768
        // 0xFFFF    -1
        // 0x7FFF    32767
        clearMemory();
        SI8(0x00, 0);
        SI8(0x80, 1);
        SI8(0xFF, 2);
        SI8(0xFF, 3);
        SI8(0xFF, 4);
        SI8(0x7F, 5);
        Assert.expectEq("lix16 load byte written by si8()", -32768, LIX16(0));
        Assert.expectEq("lix16 load byte written by si8()", -1, LIX16(2));
        Assert.expectEq("lix16 load byte written by si8()", 32767, LIX16(4));
    }

    function testsi16():void
    {
        // 0x8000    -32768
        // 0xFFFF    -1
        // 0x7FFF    32767
        clearMemory();
        SI16(0x8000, 0);
        SI16(0xFFFF, 2);
        SI16(0x7FFF, 4);
        Assert.expectEq("lix16 load bytes written by si16(0x8000)", -32768, LIX16(0));
        Assert.expectEq("lix16 load bytes written by si16(0xFFFF)", -1, LIX16(2));
        Assert.expectEq("lix16 load bytes written by si16(0x7FFF)", 32767, LIX16(4));
    }

    function testsi32():void
    {
        clearMemory();
        SI32(0x7FFF8000, 0);
        Assert.expectEq("lix16 load 1st short written by si32(0x7FFF8000)", -32768, LIX16(0));
        Assert.expectEq("lix16 load 2nd short written by si32(0x7FFF8000)", 32767, LIX16(2));
    }

    function testwriteByte():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeByte(0x00);
        mem.writeByte(0x80);
        mem.writeByte(0xFF);
        mem.writeByte(0x7F);

        Assert.expectEq("lix16 load bytes written by writeByte()", -32768, LIX16(0));
        Assert.expectEq("lix16 load bytes written by writeByte()", 32767, LIX16(2));
    }

    function testwriteInt():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeInt(2147450880);

        Assert.expectEq("lix16 load 1st short written by writeInt(2147450880)", -32768, LIX16(0));
        Assert.expectEq("lix16 load 2nd short written by writeInt(2147450880)", 32767, LIX16(2));
    }

}
