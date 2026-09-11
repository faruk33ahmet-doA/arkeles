import { useMemo } from "react";
import { cn } from "@/lib/cn";

/*
 * Not gövdesi — Sprint 3 madde 10.
 *
 * "Markdown'ı kullanıcıya ham kaynak olarak göstermek zorunda değilsin.
 *  ARKELÉS tasarımına uygun render et.
 *  ANCAK: ARKELÉS metin editörü değildir. Not gövdesi DÜZENLENEMEZ."
 *
 * Bu yüzden hafif bir OKUMA render'ı: başlıklar, listeler, alıntı, kod,
 * kalın/italik/kod-içi ve wiki bağlantıları. Markdown kütüphanesi
 * EKLENMEDİ (madde 14.1) — ihtiyacımız okuma, tam CommonMark değil.
 *
 * GÜVENLİK: hiçbir yerde `dangerouslySetInnerHTML` yok. Not içeriği
 * kullanıcının kendi verisi ama HTML olarak yorumlamak gereksiz bir
 * yüzey açardı.
 */

type Block =
  | { kind: "heading"; level: 1 | 2 | 3; text: string }
  | { kind: "paragraph"; text: string }
  | { kind: "list"; items: string[] }
  | { kind: "quote"; text: string }
  | { kind: "code"; text: string }
  | { kind: "rule" };

function parse(body: string): Block[] {
  const lines = body.split("\n");
  const blocks: Block[] = [];

  let paragraph: string[] = [];
  let list: string[] = [];
  let code: string[] | null = null;

  const flushParagraph = () => {
    if (paragraph.length > 0) {
      blocks.push({ kind: "paragraph", text: paragraph.join(" ") });
      paragraph = [];
    }
  };
  const flushList = () => {
    if (list.length > 0) {
      blocks.push({ kind: "list", items: list });
      list = [];
    }
  };
  const flushAll = () => {
    flushParagraph();
    flushList();
  };

  for (const raw of lines) {
    const line = raw.replace(/\r$/, "");
    const trimmed = line.trim();

    if (trimmed.startsWith("```") || trimmed.startsWith("~~~")) {
      if (code === null) {
        flushAll();
        code = [];
      } else {
        blocks.push({ kind: "code", text: code.join("\n") });
        code = null;
      }
      continue;
    }
    if (code !== null) {
      code.push(line);
      continue;
    }

    if (trimmed === "") {
      flushAll();
      continue;
    }
    if (trimmed === "---" || trimmed === "***") {
      flushAll();
      blocks.push({ kind: "rule" });
      continue;
    }

    const heading = /^(#{1,3})\s+(.*)$/.exec(trimmed);
    if (heading) {
      flushAll();
      blocks.push({
        kind: "heading",
        level: heading[1]!.length as 1 | 2 | 3,
        text: heading[2]!,
      });
      continue;
    }

    if (trimmed.startsWith("> ")) {
      flushAll();
      blocks.push({ kind: "quote", text: trimmed.slice(2) });
      continue;
    }

    // Görev satırları burada GÖSTERİLMEZ: kendi panelleri var ve orada
    // mutasyon yapılabilir. Burada tekrar göstermek iki kaynak yaratırdı.
    if (/^[-*+]\s+\[.\]/.test(trimmed)) {
      continue;
    }

    const bullet = /^[-*+]\s+(.*)$/.exec(trimmed);
    if (bullet) {
      flushParagraph();
      list.push(bullet[1]!);
      continue;
    }

    flushList();
    paragraph.push(trimmed);
  }

  if (code !== null) blocks.push({ kind: "code", text: code.join("\n") });
  flushAll();

  return blocks;
}

/** Satır içi işaretleri sade metne indirger — okuma için yeterli. */
function inline(text: string): string {
  return text
    .replace(/!?\[\[([^\]|#]+)(?:#[^\]|]*)?(?:\|([^\]]+))?\]\]/g, (_m, target, alias) =>
      alias || target,
    )
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/(\*\*|__)(.*?)\1/g, "$2")
    .replace(/(\*|_)(.*?)\1/g, "$2")
    .replace(/`([^`]+)`/g, "$1");
}

export function NoteBody({ body }: { body: string }) {
  const blocks = useMemo(() => parse(body), [body]);

  if (blocks.length === 0) {
    return <p className="text-base text-text-tertiary">Bu notun gövdesi boş.</p>;
  }

  return (
    <div className="flex select-text flex-col gap-4">
      {blocks.map((block, index) => {
        switch (block.kind) {
          case "heading": {
            const size =
              block.level === 1 ? "text-lg" : block.level === 2 ? "text-base" : "text-sm";
            return (
              <h3
                key={index}
                className={cn(size, "font-semibold text-text-primary")}
              >
                {inline(block.text)}
              </h3>
            );
          }
          case "paragraph":
            return (
              <p key={index} className="text-base leading-normal text-text-secondary">
                {inline(block.text)}
              </p>
            );
          case "list":
            return (
              <ul key={index} className="flex flex-col gap-2 pl-4">
                {block.items.map((item, i) => (
                  <li
                    key={i}
                    className="list-disc text-base text-text-secondary marker:text-text-tertiary"
                  >
                    {inline(item)}
                  </li>
                ))}
              </ul>
            );
          case "quote":
            return (
              <p
                key={index}
                className="border-l border-border-strong pl-4 text-base italic text-text-secondary"
              >
                {inline(block.text)}
              </p>
            );
          case "code":
            return (
              <pre
                key={index}
                className="overflow-x-auto rounded-sm bg-surface-3 p-4 text-sm text-text-secondary"
              >
                {block.text}
              </pre>
            );
          case "rule":
            return <hr key={index} className="border-border-subtle" />;
        }
      })}
    </div>
  );
}
