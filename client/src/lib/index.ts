/**
 * A video configuration.
 */
type Config = {
  resolution: [number, number];
  framerate: number;
  rotation: 0 | 90 | 180 | 270;
  device: string;
  v4l2Controls: Record<string, string>;
};

/**
 * A map of device paths to `Option` arrays.
 */
type Options = Record<string, Option[]>;

/**
 * A possible resolution and framerate combination for a device.
 */
type Option = {
  resolution: [number, number];
  framerate: number;
};

/**
 * Gets the current JSON web token stored in the 'token' cookie.
 *
 * @returns The current JWT
 */
const getToken = (): string | null => {
  const match = document.cookie.match(new RegExp('(^| )token=([^;]+)'));
  if (match) {
    return match[2];
  } else {
    return null;
  }
};

/**
 * Clears the current 'token' cookie, logging the user out.
 */
const clearToken = () => {
  document.cookie = 'token=; Max-Age=0';
};

export type { Config, Options, Option };
export { getToken, clearToken };
