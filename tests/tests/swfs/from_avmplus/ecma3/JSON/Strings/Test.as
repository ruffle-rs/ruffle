/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "JSON";
// var VERSION = "AS3";
// var TITLE   = "JSON String tests for JSON.parse and JSON.stringify";


Assert.expectEq("Unicode \\u00bb","\u00bb",JSON.parse('"\\u00bb"'));
Assert.expectEq("Unicode \\u00ab","\u00ab",JSON.parse('"\\u00ab"'));
Assert.expectEq("Unicode \\u00bf","\u00bf",JSON.parse('"\\u00bf"'));
Assert.expectEq("Unicode \\u00Ab","\u00Ab",JSON.parse('"\\u00Ab"'));
Assert.expectEq("Unicode \\u00bF","\u00bF",JSON.parse('"\\u00bF"'));
Assert.expectEq("JSON parse true",true,JSON.parse('true'));
Assert.expectEq("JSON parse false",false,JSON.parse('false'));
Assert.expectEq("JSON parse null",null,JSON.parse('null '));

Assert.expectEq("JSON parse 0",0,JSON.parse('0'));
Assert.expectEq("JSON parse 1",1,JSON.parse('1'));
Assert.expectEq("JSON parse 2",2,JSON.parse('2'));
Assert.expectEq("JSON parse 3",3,JSON.parse('3'));
Assert.expectEq("JSON parse 4",4,JSON.parse('4'));
Assert.expectEq("JSON parse 5",5,JSON.parse('5'));
Assert.expectEq("JSON parse 6",6,JSON.parse('6'));
Assert.expectEq("JSON parse 7",7,JSON.parse('7'));
Assert.expectEq("JSON parse 8",8,JSON.parse('8'));
Assert.expectEq("JSON parse 9",9,JSON.parse('9'));

Assert.expectEq("JSON.stringify('')",'""',JSON.stringify(''));
Assert.expectEq("JSON.stringify('\n\f\\u0061')",'"\\n\\f\\\\u0061"',JSON.stringify('\n\f\\u0061'));

/*
Assert.expectEq("JSON.stringify('mn\\u0001op\\u0002qr\\u0003st')",
            '"mn\\u0001op\\u0002qr\\u0003st"',
            JSON.stringify('mn\u0001op\u0002qr\u0003st'));
*/
