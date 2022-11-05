type Config = {
  resolution: [number, number];
  framerate: number;
  rotation: 0 | 90 | 180 | 270;
  device: string;
  // v4l2Options: Record<string, string>,
};

type Options = {
  [device: string]: Option[];
};

type Option = {
  resolution: [number, number];
  framerate: number;
};

export type { Config, Options, Option };
