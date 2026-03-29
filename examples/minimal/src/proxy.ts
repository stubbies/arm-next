import { NextResponse } from "next/server";
import { withArmNextProxy } from "arm-next/proxy";

export const proxy = withArmNextProxy(() => NextResponse.next());

export const config = {
  matcher: ["/((?!_next/static|_next/image|favicon.ico).*)"],
};
