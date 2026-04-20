/* -*- Mode: js; c-basic-offset: 4; indent-tabs-mode: nil; tab-width: 4 -*- */
/* vi: set ts=4 sw=4 expandtab: (add to ~/.vimrc: set modeline modelines=5) */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION:String = "RegExpObject::replace(Stringp, Stringp)";
// var VERSION:String = "";
// var TITLE:String = "Tests based on code coverage";


var testcases = getTestCases();

function getTestCases() : Array
{
    var array:Array = new Array();
    var item:int = 0;

    var str:String;
    var pattern:RegExp;

    str = "one-two";
    pattern = /(\w+)-(\w+)/g;
    array[item++] = Assert.expectEq( 'str.replace(pattern, "$2-$1")', "two-one", str.replace(pattern, "$2-$1") );

    pattern = /(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)/g;
    str = "one-two-three-four-five-six-seven-eight-nine-ten";
    array[item++] = Assert.expectEq( 'str.replace(pattern, "$9-$8-$7-$6-$5-$4-$3-$2-$1")', "nine-eight-seven-six-five-four-three-two-one", str.replace(pattern, "$9-$8-$7-$6-$5-$4-$3-$2-$1") );

    pattern = /(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)-(\w+)/g;
    str = "one-two-three-four-five-six-seven-eight-nine-ten";
    array[item++] = Assert.expectEq( 'str.replace(pattern, "$10-$1")', "ten-one", str.replace(pattern, "$10-$1") );

    str = "one-two";
    pattern = /(\w+)-(\w+)/g;
    array[item++] = Assert.expectEq( 'str.replace(pattern, "$2-$A")', "two-$A", str.replace(pattern, "$2-$A") );

    str = "one-two";
    pattern = /(\w+)-(\w+)/g;
    array[item++] = Assert.expectEq( 'str.replace(pattern, "$02-$01")', "two-one", str.replace(pattern, "$02-$01") );

    str = "abc-123-abc";
    pattern = /(?P<mygroup>abc)-123-(?P=mygroup)/g;
    array[item++] = Assert.expectEq( "pattern.exec(str)", "abc", pattern.exec(str)[1] );

    str = "abc-456-abc";
    pattern = /(?P<mygroup>abc)-123-(?P=mygroup)/g;
    array[item++] = Assert.expectEq( "pattern.exec(str)", null, pattern.exec(str) );

    return ( array );
}

