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
// var TITLE   = "Testing nested try block with multiple catch blocks, the inner try and multiple catch block inside the fourth catch block of the outer try and multiple catch blocks";  // Provide ECMA section title or a description
var BUGNUMBER = "";



thisError = "no error";
thisError1="no error";
try{
   throw new URIError();
   }catch(eo:ReferenceError){
       thisError1 = "This is outer Reference error:"+"  "+eo.toString();
       //print(thisError1);
   }catch(eo1:TypeError){
       thisError1="This is outer TypeError:"+eo1.toString();
       //print(thisError1);
   }catch(eo2:ArgumentError){
       thisError1="This is outer Argument Error:"+eo2.toString();
       //print(thisError1);
   }catch(eo3:URIError){
       thisError1="This is outer URI Error:"+eo3.toString();
       try {
           throw new TypeError();
           }catch(ei:TypeError){
               thisError="This is Inner Type Error:"+ei.toString();
               //print(thisError);
           }catch(ei1:ReferenceError){
               thisError="Inner reference error:"+ei1.toString();
           }catch(ei2:URIError){
               thisError="This is inner URI Error:"+ei2.toString();
           }catch(ei3:EvalError){
               thisError="This is inner Eval Error:"+ei3.toString();
           }catch(ei4:RangeError){
               thisError="This is inner Range Error:"+ei4.toString();
           }catch(ei5:SecurityError){
               thisError="This is inner Security Error!!!"+ei5.toString();
           }catch(ei6:ArgumentError){
               thisError="This is inner Argument Error"+ei6.toString();
           }finally{
               Assert.expectEq( "Testing Nested try block with multiple catch block inside the fourth catch block of the outer try block","This is inner finally:This is Inner Type Error:TypeError","This is inner finally:"+thisError );
            }
   }catch(eo4:EvalError){
       thisError1="This is outer Eval Error"+eo4.toString();
   }catch(eo5:RangeError){
       thisError1="This is outer Range Error"+eo5.toString();
   }catch(eo6:SecurityError){
       thisError1="This is outer Security Error!!!"+eo6.toString();
   }catch(eo7:Error){
       thisError1="This is outer Error:"+eo7.toString();
   }finally{
           Assert.expectEq( "Testing Nested try block with multiple catch block inside the fourth catch block of the outer try block", "This is outer finally:This is outer URI Error:URIError","This is outer finally:"+ thisError1 );
    }
 


              // displays results.
