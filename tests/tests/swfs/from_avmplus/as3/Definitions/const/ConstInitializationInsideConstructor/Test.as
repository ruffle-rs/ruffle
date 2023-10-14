/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip; public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions\const";                  // provide a document reference (ie, ECMA section)
// var VERSION = "ActionScript 3.0";           // Version of JavaScript or ECMA
// var TITLE   = "initialization of const inside function";       // Provide ECMA section title or a description
var BUGNUMBER = "";


class Person {
    const name:String;
    
    function Person(theName:String)
    {
        name = theName;
    }
    
    function getName():String
    {
        return name;
    }
}

var bob:Person = new Person("bob");

Assert.expectEq("Initialize instance const inside constructor", "bob", bob.getName());

