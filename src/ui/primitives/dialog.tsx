/*
 * Davranış primitifi — Anayasa madde 31.1.
 *
 * Radix Dialog'un ince sarmalayıcısı. Buradan aldığımız şey DAVRANIŞ:
 * focus trap, Esc ile kapanma, scroll lock, aria bağlantıları.
 * Görsel kararlar (madde 31.3) çağıran tarafta, token'larla verilir —
 * bu dosya renk/gölge/radius TANIMLAMAZ.
 */

import * as DialogPrimitive from "@radix-ui/react-dialog";

export const Dialog = DialogPrimitive.Root;
export const DialogTrigger = DialogPrimitive.Trigger;
export const DialogPortal = DialogPrimitive.Portal;
export const DialogOverlay = DialogPrimitive.Overlay;
export const DialogContent = DialogPrimitive.Content;
export const DialogTitle = DialogPrimitive.Title;
export const DialogDescription = DialogPrimitive.Description;
export const DialogClose = DialogPrimitive.Close;
