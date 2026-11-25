import {useEffect,useState} from 'react';
import {Platform} from 'react-native';

type Viewport={width:number;height:number;left:number;top:number};
/** Desktop/browser zoom can make the visible area smaller than RN Web's
 * scale-adjusted Dimensions. Keep dialogs inside the part the user can see. */
export function useVisibleViewport(width:number,height:number):Viewport {
  const [visible,setVisible]=useState<Viewport|null>(null);
  useEffect(()=>{
    if(Platform.OS!=='web')return;
    const viewport=window.visualViewport;
    const update=()=>{
      const next={width:viewport?.width??window.innerWidth,height:viewport?.height??window.innerHeight,left:viewport?.offsetLeft??0,top:viewport?.offsetTop??0};
      if(next.width<=0||next.height<=0)return;
      setVisible(old=>old&&old.width===next.width&&old.height===next.height&&old.left===next.left&&old.top===next.top?old:next);
    };
    update();
    window.addEventListener('resize',update);
    viewport?.addEventListener('resize',update);viewport?.addEventListener('scroll',update);
    return ()=>{window.removeEventListener('resize',update);viewport?.removeEventListener('resize',update);viewport?.removeEventListener('scroll',update);};
  },[]);
  return visible??{width,height,left:0,top:0};
}
