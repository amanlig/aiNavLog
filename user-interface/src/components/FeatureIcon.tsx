import Svg, { Circle, Path, Rect } from 'react-native-svg';

export type Feature = 'electrical' | 'engine' | 'routes';
export function FeatureIcon({ feature, size = 116 }: { feature: Feature; size?: number }) {
  return <Svg width={size} height={size} viewBox="0 0 128 128" fill="none" stroke={feature === 'engine' ? '#103e5c' : '#064b5a'} strokeWidth={5} strokeLinecap="round" strokeLinejoin="round">
    {feature === 'electrical' ? <>
      <Path d="M30 35V24h15v11m38 0V24h15v11" /><Rect x={17} y={35} width={94} height={67} rx={5} />
      <Path d="M29 54h14m-7-7v14m50-7h13" /><Path d="m70 44-22 33h15l-5 18 24-33H66Z" fill="#064b5a" strokeWidth={2} />
    </> : feature === 'engine' ? <Path d="M17 51h16V38h22V25h19v13h11l13 17h13v33H95v15H53L36 86H22V51Zm-7 0v36m0-18h12m89-14h9v33h-9M46 25h36" /> : <>
      <Path d="M45 79c0 13-16 31-16 31S13 92 13 79a16 16 0 1 1 32 0Zm68-52c0 13-16 31-16 31S81 40 81 27a16 16 0 1 1 32 0Z" />
      <Circle cx={29} cy={79} r={4} /><Circle cx={97} cy={27} r={4} /><Path d="M40 110c44 0 18-46 49-46" strokeDasharray="5 9" />
    </>}
  </Svg>;
}
export function CompassIcon() {
  return <Svg width={44} height={44} viewBox="0 0 64 64" fill="none" stroke="white" strokeWidth={2}>
    <Circle cx={32} cy={32} r={28} /><Path d="M32 5v54M5 32h54M20 44l8-16 16-8-8 16Z" /><Path d="m32 10 6 23-6-3-6 3Z" fill="white" />
  </Svg>;
}
