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
// var TITLE   = "Testing try block with multiple catch blocks, the  catch block with parameter of type ReferenceError catching the reference error ";  // Provide ECMA section title or a description
var BUGNUMBER = "";



thisError = "no error";
       
try {
     throw new ReferenceError();
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
    }catch(e4:TypeError){
       thisError="This is Type Error";
    }catch(e5:SecurityError){
       thisError="This is security Error";
    }catch(e6:Error){
      thisError = e6.toString();
    }finally{//print(thisError);
Assert.expectEq( "Testing try block with throw statement", "This is Reference Error"        ,thisError );
     }


              // displays results.
