/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


// var SECTION = "Expressions";       // provide a document reference (ie, Actionscript section)
// var VERSION = "AS 3.0";        // Version of ECMAScript or ActionScript
// var TITLE   = "qualified references";       // Provide ECMA section title or a description
var BUGNUMBER = "";





import poo.*;

import com.adobe.test.Assert;
var f:foo = new foo();
Assert.expectEq( "f.teamName = 'Macromedia'", "Macromedia", f.teamName );
Assert.expectEq( "f.public::teamName = 'Macromedia'", "Macromedia", f.public::teamName );
Assert.expectEq( "f.Baseball::teamName = 'Giants'", "Giants", f.Baseball::teamName );
Assert.expectEq( "f.Football::teamName = 'Chargers'", "Chargers", f.Football::teamName );
Assert.expectEq( "f.Basketball::teamName = 'Kings'", "Kings", f.Basketball::teamName );
Assert.expectEq( "f.Hockey::teamName = 'Sharks'", "Sharks", f.Hockey::teamName );

Assert.expectEq( "f.Hockey::teamColor = 'yellow'", "yellow", f.Hockey::teamColor );

              // displays results.
