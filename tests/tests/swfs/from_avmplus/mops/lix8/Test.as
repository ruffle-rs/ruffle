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
//     var TITLE:String   = "lix8";


    Assert.expectError("lix8(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH) prior to initMemory()",
                 Utils.RANGEERROR+1506,
                 function(){ LIX8(ApplicationDomain.MIN_DOMAIN_MEMORY_LENGTH); });

    initMemory();
    // Get a handle to the domainMemory after it is initialized
    var mem:ByteArray = ApplicationDomain.currentDomain.domainMemory;

    Assert.expectError("lix8(-1)", Utils.RANGEERROR+1506, function(){ LIX8(-1); });
    Assert.expectError("lix8(mem.length)", Utils.RANGEERROR+1506, function(){ LIX8(mem.length); });
    Assert.expectEq("lix8(mem.length-1)", 0, LIX8(mem.length-1));

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
        clearMemory();
        SI8(0x7F, 0);
        SI8(0x80, 1);
        SI8(0xFF, 2);
        Assert.expectEq("lix8 load byte written by si8(0x7F)", 127, LIX8(0));
        Assert.expectEq("lix8 load byte written by si8(0x80)", -128, LIX8(1));
        Assert.expectEq("lix8 load byte written by si8(0xFF)", -1, LIX8(2));
    }

    function testsi16():void
    {
        clearMemory();
        SI16(0x7F80, 0);
        Assert.expectEq("lix8 load 1st byte written by si16(0x7F80)", 127, LIX8(1));
        Assert.expectEq("lix8 load 2nd byte written by si16(0x7F80)", -128, LIX8(0));
    }

    function testsi32():void
    {
        clearMemory();
        SI32(0x007F80FF, 0);
        Assert.expectEq("lix8 load 1st byte written by si32(0x007F80FF)", 0, LIX8(3));
        Assert.expectEq("lix8 load 2nd byte written by si32(0x007F80FF)", 127, LIX8(2));
        Assert.expectEq("lix8 load 3rd byte written by si32(0x007F80FF)", -128, LIX8(1));
        Assert.expectEq("lix8 load 4th byte written by si32(0x007F80FF)", -1, LIX8(0));
    }

    function testwriteByte():void
    {
        clearMemory();
        mem.position = 0;
        mem.writeByte(0);
        mem.writeByte(127);
        mem.writeByte(128);
        mem.writeByte(255);

        Assert.expectEq("lix8 load byte written by writeByte(0)", 0, LIX8(0));
        Assert.expectEq("lix8 load byte written by writeByte(127)", 127, LIX8(1));
        Assert.expectEq("lix8 load byte written by writeByte(128)", -128, LIX8(2));
        Assert.expectEq("lix8 load byte written by writeByte(255)", -1, LIX8(3));
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

        Assert.expectEq("lix8 load 1st byte written by writeInt(2147473647)", 127, LIX8(3));
        Assert.expectEq("lix8 load 2nd byte written by writeInt(2147473647)", -1, LIX8(2));
        Assert.expectEq("lix8 load 3rd byte written by writeInt(2147473647)", -40, LIX8(1));
        Assert.expectEq("lix8 load 4th byte written by writeInt(2147473647)", -17, LIX8(0));
    }

}
