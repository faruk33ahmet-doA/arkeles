/*
 * Davranış primitifi — Anayasa madde 31.1.
 *
 * cmdk: liste filtreleme, klavye navigasyonu, aria-activedescendant.
 * Bunu sıfırdan yazmak haftalar alır ve hatalı olur (madde 31.2).
 * Görsel katman çağıranda (madde 31.3).
 */
import { Command as CommandPrimitive } from "cmdk";

export const Command = CommandPrimitive;
export const CommandInput = CommandPrimitive.Input;
export const CommandList = CommandPrimitive.List;
export const CommandEmpty = CommandPrimitive.Empty;
export const CommandGroup = CommandPrimitive.Group;
export const CommandItem = CommandPrimitive.Item;
export const CommandSeparator = CommandPrimitive.Separator;
