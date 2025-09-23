/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Expressions";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";        // Version of ECMAScript or ActionScript
// var TITLE   = "Logical Assignment";       // Provide ECMA section title or a description



// Logical AND
// Assigns expression1 the value of expression1 && expression2. For example, the following two statements are equivalent:
//  x &&= y;
//  x = x && y;

var x = true;
var y = true;

x &&= y;
Assert.expectEq('Logical AND Assignment: x=true; y=true; x &&= y; x=', true, x);

x=true;
y=false;
x &&= y;
Assert.expectEq('Logical AND Assignment: x=true; y=false; x &&= y; x=', false, x);

x=false;
y=false;
x &&= y;
Assert.expectEq('Logical AND Assignment: x=false; y=false; x &&= y; x=', false, x);

x=false;
y=true;
x &&= y;
Assert.expectEq('Logical AND Assignment: x=false; y=true; x &&= y; x=', false, x);

// using var and const
x = true;
x &&= true;
Assert.expectEq('Logical AND Assignment: x=true; x &&= true; x=', true, x);

x = true;
x &&= false;
Assert.expectEq('Logical AND Assignment: x=true; x &&= false; x=', false, x);

x = false;
x &&= false;
Assert.expectEq('Logical AND Assignment: x=false; x &&= false; x=', false, x);

x = false;
x &&= true;
Assert.expectEq('Logical AND Assignment: x=false; x &&= true; x=', false, x);

// Logical OR
// Assigns expression1 the value of expression1 || expression2. For example, the following two statements are equivalent:
//  x ||= y;
//  x = x || y;

x=true;
y=true;
x ||= y;
Assert.expectEq('Logical OR Assignment: x=true; y=true; x ||= y; x=', true, x);

x=true;
y=false;
x ||= y;
Assert.expectEq('Logical OR Assignment: x=true; y=false; x ||= y; x=', true, x);

x=false;
y=false;
x ||= y;
Assert.expectEq('Logical OR Assignment: x=false; y=false; x ||= y; x=', false, x);

x=false;
y=true;
x ||= y;
Assert.expectEq('Logical OR Assignment: x=false; y=true; x ||= y; x=', true, x);

// using var and const
x=true;
x ||= true;
Assert.expectEq('Logical OR Assignment: x=true; x ||= true; x=', true, x);

x=true;
x ||= false;
Assert.expectEq('Logical OR Assignment: x=true; x ||= false; x=', true, x);

x=false;
x ||= false;
Assert.expectEq('Logical OR Assignment: x=false; x ||= false; x=', false, x);

x=false;
x ||= true;
Assert.expectEq('Logical OR Assignment: x=false; x ||= true; x=', true, x);

              // displays results.