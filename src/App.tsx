import { Providers } from "@/app/Providers";
import { AppShell } from "@/app/AppShell";

export default function App() {
  return (
    <Providers>
      <AppShell />
    </Providers>
  );
}
