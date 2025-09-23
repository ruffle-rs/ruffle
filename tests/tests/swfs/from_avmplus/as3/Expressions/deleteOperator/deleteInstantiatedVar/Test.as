/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Expressions";        // provide a document reference (ie, Actionscript section)
// var VERSION = "AS3";                // Version of ECMAScript or ActionScript
// var TITLE   = "delete operator";    // Provide ECMA section title or a description
var BUGNUMBER = "";


// create the new object "account"
var account:Object = new Object();

// assign property name to the account
account.name = "Jon";

// delete the property
Assert.expectEq("delete instantiated object's var", true, delete account.name);
Assert.expectEq("account.name", undefined, account.name);


              // displays results.
