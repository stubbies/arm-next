import { createArmNextProxy } from "arm-next/proxy";

export const proxy = createArmNextProxy();

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
