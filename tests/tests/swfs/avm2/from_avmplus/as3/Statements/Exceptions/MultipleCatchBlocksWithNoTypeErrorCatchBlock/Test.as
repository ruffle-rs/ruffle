/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// var SECTION = "Definitions";           // provide a document reference (ie, ECMA section)
// var VERSION = "AS3";                   // Version of JavaScript or ECMA
// var TITLE   = "Testing try block with multiple catch blocks, the  catch block with parameter of type Error catching the type Error as there is no catch block with parameter of type TypeError";  // Provide ECMA section title or a description
var BUGNUMBER = "";



thisError = "no error";
       
try {
     throw new TypeError();
    } catch(e:ReferenceError){
       thisError="This is Reference Error";
    }catch(e1:ArgumentError){
       thisError="This is Argument Error";
    }catch(e2:URIError){
       thisError="This is URI Error";
    }catch(e3:EvalError){
       thisError="This is Eval Error";
    }catch(e4:RangeError){
       thisError="This is Range Error";
    }catch(e5:SecurityError){
       thisError="This is security Error";
    }catch(e6:Error){
      thisError = e6.toString();
    }finally{
       Assert.expectEq( "Testing try block with throw statement", "Error: TypeError"        ,Error(thisError)+""  );
     }


              // displays results.
