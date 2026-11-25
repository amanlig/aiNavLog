import {useEffect,useRef} from 'react';
import {Platform,type GestureResponderEvent} from 'react-native';

type Point={x:number;y:number};
/** Claim only a deliberate one-finger horizontal drag. Taps/holds, mouse input,
 * vertical scrolling and multi-touch retain their ordinary diagram behavior. */
export function useViewSwipe(enabled:boolean,onSwipe:(direction:1|-1)=>void) {
  const origin=useRef<Point|null>(null);
  const delta=useRef<Point>({x:0,y:0});
  const cancelled=useRef(false);
  const blockPress=useRef(false);
  const timer=useRef<ReturnType<typeof setTimeout>|null>(null);
  useEffect(()=>()=>{if(timer.current)clearTimeout(timer.current);},[]);
  function start(event:GestureResponderEvent) {
    const native=event.nativeEvent as typeof event.nativeEvent & {type?:string;pointerType?:string};
    if(!enabled||native.touches.length!==1||(Platform.OS==='web'&&!native.type?.startsWith('touch')&&native.pointerType!=='touch')) {
      origin.current=null;cancelled.current=true;return false;
    }
    origin.current={x:native.touches[0].pageX,y:native.touches[0].pageY};
    delta.current={x:0,y:0};cancelled.current=false;
    return false;
  }
  function measure(event:GestureResponderEvent) {
    if(event.nativeEvent.touches.length!==1){cancelled.current=true;return;}
    if(origin.current)delta.current={x:event.nativeEvent.touches[0].pageX-origin.current.x,y:event.nativeEvent.touches[0].pageY-origin.current.y};
  }
  function wants(event:GestureResponderEvent) {
    measure(event);
    if(!enabled||!origin.current||cancelled.current)return false;
    const {x,y}=delta.current;
    if(Math.abs(y)>24&&Math.abs(y)>Math.abs(x)){cancelled.current=true;return false;}
    return Math.abs(x)>24&&Math.abs(x)>Math.abs(y)*1.5;
  }
  function finish(commit:boolean) {
    const {x,y}=delta.current;
    const swipe=commit&&enabled&&!cancelled.current&&Math.abs(x)>=60&&Math.abs(x)>Math.abs(y)*1.5;
    origin.current=null;
    if(timer.current)clearTimeout(timer.current);
    timer.current=setTimeout(()=>{blockPress.current=false;},300);
    if(swipe)onSwipe(x<0?1:-1);
  }
  return {blockPress,handlers:{
    onStartShouldSetResponderCapture:start,
    onMoveShouldSetResponderCapture:wants,
    onResponderGrant:()=>{blockPress.current=true;},
    onResponderMove:(event:GestureResponderEvent)=>{measure(event);event.preventDefault();},
    onResponderStart:(event:GestureResponderEvent)=>{if(event.nativeEvent.touches.length!==1)cancelled.current=true;},
    onResponderRelease:()=>finish(true),
    onResponderTerminate:()=>finish(false),
    onResponderTerminationRequest:()=>true,
  }};
}
