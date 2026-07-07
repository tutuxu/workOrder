import { commands } from "../bindings";
import type { Attachment, OwnerType } from "../types";

export async function listAttachments(
  ownerType: OwnerType,
  ownerId: number,
): Promise<Attachment[]> {
  return commands.listAttachments(ownerType, ownerId);
}

export async function addAttachmentFromFile(
  ownerType: OwnerType,
  ownerId: number,
  sourcePath: string,
): Promise<Attachment> {
  return commands.addAttachmentFromFile(ownerType, ownerId, sourcePath);
}

export async function addAttachmentFromBytes(
  ownerType: OwnerType,
  ownerId: number,
  fileName: string,
  mimeType: string,
  data: Uint8Array,
): Promise<Attachment> {
  return commands.addAttachmentFromBytes(
    ownerType,
    ownerId,
    fileName,
    mimeType,
    Array.from(data),
  );
}

export async function deleteAttachment(id: number): Promise<void> {
  return commands.deleteAttachment(id).then(() => undefined);
}

export async function pickAttachmentFile(): Promise<string | null> {
  return commands.pickAttachmentFile();
}
