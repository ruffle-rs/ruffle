package {
	public class Test {}
}

trace("//2 === \"2\"");
trace(2 === "2");

trace("//2 === 2");
trace(2 === 2);

trace("//2 === 5");
trace(2 === 5);

trace("//true === true");
trace(true === true);

trace("//false === false");
trace(false === false);

trace("//true === false");
trace(true === false);

trace("//1 === true");
trace(1 === true);

trace("//0 === false");
trace(0 === false);

trace("//\"abc\" === \"abc\"");
trace("abc" === "abc");

trace("//0 === undefined");
trace(0 === undefined);

trace("//undefined === undefined");
trace(undefined === undefined);

trace("//NaN === NaN");
trace(NaN === NaN);

trace("//undefined === NaN");
trace(undefined === NaN);

trace("//0 === null");
trace(0 === null);

trace("//null === null");
trace(null === null);

trace("//undefined === null");
trace(undefined === null);

trace("//NaN === null");
trace(NaN === null);
