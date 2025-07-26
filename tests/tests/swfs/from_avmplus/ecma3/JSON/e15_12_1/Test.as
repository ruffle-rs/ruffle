/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;


function removeExceptionDetail(s:String) {
    var fnd=s.indexOf(" ");
    if (fnd>-1) {
        if (s.indexOf(':',fnd)>-1) {
                s=s.substring(0,s.indexOf(':',fnd));
        }
    }
    return s;
}

function sortObject(o:Object) {
    var keys=[];
    var key;
    for ( key in o ) {
        if (o[key]===undefined) {
           continue;
        }
        keys[keys.length]=key;
    }
    keys.sort();
    var ret="{";
    var value;
    for (var i in keys) {
        key=keys[i];
        value=o[key];
        if (value is String) {
            value='"'+value+'"';
        } else if (value is Array) {
            value='['+value+']';
        } else if (value is Object) {
        }
        ret += '"'+key+'":'+value+",";
    }
    ret=ret.substring(0,ret.length-1);
    ret+="}";
    return ret;
}

//package {

//     var SECTION = "15.2.1";
//     var VERSION = "ECMA_3";
//     var TITLE   = "JSON ecma-262 testcases";


    // set the Object.toString and Array.toString to show each property value
    // instead of [Object object] for testing
    var oldObject = Object.prototype.toString;
    var oldArray = Array.prototype.toString;

    function jformat(x) {
        if (x is String)
            return '"' + x + '"';
        else
            return x;
    }

    Object.prototype.toString = (function () {
        var s = "{";
        var first = true;
        for ( var i in this ) {
            if (this.hasOwnProperty(i)) {
                if (!first)
                    s += ",";
                s += jformat(String(i)) + ":" + jformat(this[i]);
                first = false;
            }
        }
        return s + "}";
    });
    Array.prototype.toString = (function () {
        var s = "[";
        var first = true;
        for ( var i=0 ; i < this.length ; i++ ) {
            if (!first)
                s += ",";
            first = false;
            if (!this.hasOwnProperty(i))
                continue;
            s += jformat(this[i]);
        }
        return s + "]";
    });


    // 15.12.1-1-0-1: syntax error if multiple tokens
    parseException="no exception";
    try {
        JSON.parse('12\t\r\n 34'); // should produce a syntax error as whitespace results in two tokens
    } catch (e) {
        parseException=removeExceptionDetail(removeExceptionDetail(e.toString()));
    }
    Assert.expectEq("15.12.1-1-0-1: The JSON lexical grammar treats whitespace as a token separator","SyntaxError: Error #1132",parseException);

    // 15.12.1-1-0-1: syntax error <vt> is not valid
    vtException="no exception";
    try {
        JSON.parse('\u000b1234'); // should produce a syntax error <VT>
    } catch (e) {
        vtException=removeExceptionDetail(removeExceptionDetail(e.toString())); // treat any exception as a pass, other tests ensure that JSON.parse throws SyntaxError exceptions
    }
    Assert.expectEq("15.12.1-1-0-2: <VT> is not valid JSON whitespace as specified by the production JSONWhitespace.","SyntaxError: Error #1132",vtException);

    //<FF> is not valid JSON whitespace as specified by the production JSONWhitespace.
    var ffException="no exception";
    try {
        JSON.parse('\u000c1234'); // should produce a syntax error <FF>
    } catch (e) {
        ffException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-0-3: <FF> is not valid JSON whitespace as specified by the production JSONWhitespace.","SyntaxError: Error #1132",ffException);

    //<NBSP> is not valid JSON whitespace as specified by the production JSONWhitespace.
    var nbspException="no exception";
    try {
        JSON.parse('\u00a01234'); // should produce a syntax error <NBSP>
    } catch (e) {
        nbspException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-0-4: <NBSP> is not valid JSON whitespace as specified by the production JSONWhitespace.","SyntaxError: Error #1132",nbspException);

    //<ZWSPP> is not valid JSON whitespace as specified by the production JSONWhitespace.
    var zwsppException="no exception";
    try {
        JSON.parse('\u200b1234'); // should produce a syntax error <ZWSPP>
    } catch (e) {
        zwsppException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-0-5: <ZWSPP> is not valid JSON whitespace as specified by the production JSONWhitespace.","SyntaxError: Error #1132",zwsppException);

    //<BOM> is not valid JSON whitespace as specified by the production JSONWhitespace.
    var bomException="no exception";
    try {
        JSON.parse('\ufeff1234'); // should produce a syntax error <BOM>
    } catch (e) {
        bomException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-0-6: <BOM> is not valid JSON whitespace as specified by the production JSONWhitespace.","SyntaxError: Error #1132",bomException);

    // other category z spaces are not valid JSON whitespace as specified by the production JSONWhitespace.
    otherException="no exception";
    try {
        // the following should produce a syntax error
        JSON.parse('\u1680\u180e\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a\u202f\u205f\u30001234');
    } catch (e) {
        otherException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-0-7: other category z spaces are not valid JSON whitespace as specified by the production JSONWhitespace.",
                "SyntaxError: Error #1132",otherException);

    // U+2028 and U+2029 are not valid JSON whitespace as specified by the production JSONWhitespace
    u2028Exception="no exception";
    try {
        JSON.parse('\u2028\u20291234'); // should produce a syntax error
    } catch (e) {
        u2028Exception=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-0-8: U+2028 and U+2029 are not valid JSON whitespace as specified by the production JSONWhitespace",
                "SyntaxError: Error #1132",u2028Exception);

    Assert.expectEq("15.12.1-1-0-9: Whitespace characters can appear before/after any JSONtoken",
                "{\"prop2\":[[true,null,123.456]],\"property\":{}}",
                sortObject(JSON.parse('\t\r \n{\t\r \n'+
                   '"property"\t\r \n:\t\r \n{\t\r \n}\t\r \n,\t\r \n' +
                   '"prop2"\t\r \n:\t\r \n'+
                        '[\t\r \ntrue\t\r \n,\t\r \nnull\t\r \n,123.456\t\r \n]'+
                     '\t\r \n}\t\r \n')));

    Assert.expectEq("15.12.1-1-g1-1: The JSON lexical grammar treats <TAB> as a whitespace character",
                1234,
                JSON.parse('\t1234'));

    // <TAB> should produce a syntax error as whitespace results in two tokens
    tabException="no exception";
    try {
        JSON.parse('12\t34');
    } catch (e) {
        tabException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g1-1: <TAB> should produce a syntax error as whitespace results in two tokens",
                "SyntaxError: Error #1132",tabException);

    Assert.expectEq("15.12.1-1-g1-2: The JSON lexical grammar treats <CR> as a whitespace character",
                1234,
                JSON.parse('\r1234'));

    // <CR> should produce a syntax error as whitespace results in two tokens
    crException="no exception";
    try {
        JSON.parse('12\r34');
    } catch (e) {
        crException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g1-2: <CR> should produce a syntax error as whitespace results in two tokens",
                "SyntaxError: Error #1132",crException);

    Assert.expectEq("15.12.1-1-g1-3: The JSON lexical grammar treats <LF> as a whitespace character",
                1234,
                JSON.parse('\n1234'));

    // <LF> should produce a syntax error as whitespace results in two tokens
    lfException="no exception";
    try {
        JSON.parse('12\n34');
    } catch (e) {
        lfException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g1-3: <LF> should produce a syntax error as whitespace results in two tokens",
                "SyntaxError: Error #1132",lfException);

    Assert.expectEq("15.12.1-1-g1-4: The JSON lexical grammar treats <SP> as a whitespace character",
                1234,
                JSON.parse(' 1234'));

    // <SP> should produce a syntax error as whitespace results in two tokens
    spException="no exception";
    try {
        JSON.parse('12 34');
    } catch (e) {
        spException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g1-4: <SP> should produce a syntax error as whitespace results in two tokens",
                "SyntaxError: Error #1132",spException);

    Assert.expectEq("15.12.1-1-g2-1: JSONStrings can be written using double quotes","abc",JSON.parse('"abc"'));

    singlequoteException="no exception";
    try {
        JSON.parse("'abc'");
    } catch (e) {
        singlequoteException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g2-2: JSONString may not be delimited by single quotes","SyntaxError: Error #1132",singlequoteException);

    unicodequoteException="no exception";
    try {
        JSON.parse("\\u0022abc\\u0022");
    } catch (e) {
        unicodequoteException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g2-3: A JSONString may not be delimited by Uncode escaped quotes","SyntaxError: Error #1132",unicodequoteException);

    quoteException="no exception";
    try {
        JSON.parse('"ab'+"c'");
    } catch (e) {
        quoteException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g2-4: A JSONString must both begin and end with double quotes","SyntaxError: Error #1132",quoteException);

    Assert.expectEq("15.12.1-1-g2-5: A JSONStrings can contain no JSONStringCharacters (Empty JSONStrings)","",JSON.parse('""'));

    illegalunicodeException="no exception";
    try {
        JSON.parse('"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"');
    } catch (e) {
        illegalunicodeException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g4-1: The JSON lexical grammar does not allow a JSONStringCharacter to be any of the Unicode characters U+0000 thru U+0007",
                "SyntaxError: Error #1132",illegalunicodeException);

    illegalunicode2Exception="no exception";
    try {
        JSON.parse('"\u0008\u0009\u000a\u000b\u000c\u000d\u000e\u000f"');
    } catch (e) {
        illegalunicode2Exception=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g4-2: The JSON lexical grammar does not allow a JSONStringCharacter to be any of the Unicode characters U+0008 thru U+000F",
                "SyntaxError: Error #1132",illegalunicode2Exception);

    illegalunicode3Exception="no exception";
    try {
        JSON.parse('"\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"');
    } catch (e) {
        illegalunicode3Exception=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g4-3: The JSON lexical grammar does not allow a JSONStringCharacter to be any of the Unicode characters U+0010 thru U+0017",
                "SyntaxError: Error #1132",illegalunicode3Exception);

    illegalunicode4Exception="no exception";
    try {
        JSON.parse('"\u0018\u0019\u001a\u001b\u001c\u001d\u001e\u001f"');
    } catch (e) {
        illegalunicode4Exception=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g4-4: The JSON lexical grammar does not allow a JSONStringCharacter to be any of the Unicode characters U+0018 thru U+001F",
                "SyntaxError: Error #1132",illegalunicode4Exception);

    try {
        Assert.expectEq("15.12.1.1-g5-1: The JSON lexical grammar allows Unicode escape sequences in a JSONString","X",JSON.parse('"\\u0058"'));
    } catch (e) {
        Assert.expectEq("15.12.1.1-g5-1: The JSON lexical grammar allows Unicode escape sequences in a JSONString","no exception",e.toString());
    }

    illegalunicode5Exception="no exception";
    try {
        JSON.parse('"\\u005"');
    } catch (e) {
        illegalunicode5Exception=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g5-2: A JSONStringCharacter UnicodeEscape may not have fewer than 4 hex characters",
                "SyntaxError: Error #1132",illegalunicode5Exception);

    nonhexunicodeException="no exception";
    try {
        JSON.parse('"\\u0X50"');
    } catch (e) {
        nonhexunicodeException=removeExceptionDetail(e.toString());
    }
    Assert.expectEq("15.12.1-1-g5-3: A JSONStringCharacter UnicodeEscape may not include any non=hex characters",
                "SyntaxError: Error #1132",nonhexunicodeException);

    Assert.expectEq("15.12.1-1-g6-1: The JSON lexical grammer allows '/' as a JSONEscapeCharacter after '\\' in a JSONString",'/',JSON.parse('"\\/"'));
    Assert.expectEq("15.12.1-1-g6-2: The JSON lexical grammer allows '\' as a JSONEscapeCharacter after '\\' in a JSONString",'\\',JSON.parse('"\\\\"'));
    Assert.expectEq("15.12.1-1-g6-3: The JSON lexical grammer allows 'b' as a JSONEscapeCharacter after '\\' in a JSONString",'\b',JSON.parse('"\\b"'));
    Assert.expectEq("15.12.1-1-g6-4: The JSON lexical grammer allows 'f' as a JSONEscapeCharacter after '\\' in a JSONString",true,JSON.parse('"\\f"')=='\f');
    Assert.expectEq("15.12.1-1-g6-5: The JSON lexical grammer allows 'n' as a JSONEscapeCharacter after '\\' in a JSONString",true,JSON.parse('"\\n"')=='\n');
    Assert.expectEq("15.12.1-1-g6-6: The JSON lexical grammer allows 'r' as a JSONEscapeCharacter after '\\' in a JSONString",true,JSON.parse('"\\r"')=='\r');
    Assert.expectEq("15.12.1-1-g6-7: The JSON lexical grammer allows 't' as a JSONEscapeCharacter after '\\' in a JSONString",'\t',JSON.parse('"\\t"'));


    Object.prototype.toString = oldObject;
    Array.prototype.toString = oldArray;
//}
