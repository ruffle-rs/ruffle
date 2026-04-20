/* -*- c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


// var SECTION = "regress_513020";
// var VERSION = "AS3";
// var TITLE   = "[Regexp] String.match with global flag returns null when nothing is found";
// var bug = "513020";


import com.adobe.test.Assert;
var line:String = "aaa";
var GIpattern:RegExp = /bbb/gi;
var Ipattern:RegExp = /bbb/i;

function MyMatch(myPattern:RegExp)
{
    var result = line.match(myPattern);
    if(result == null)
        return "null";
    else
        return "not null";
}

Assert.expectEq(
    "regex non-match returns not null with /gi",
    "not null",
    MyMatch(GIpattern));

Assert.expectEq(
    "regex non-match returns null with /i",
    "null",
    MyMatch(Ipattern));


